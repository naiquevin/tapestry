use crate::error::Error;
use crate::metadata::Metadata;
use crate::placeholder::Placeholder;
use minijinja::{context, path_loader, Environment};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::convert::From;
use std::path::Path;

pub fn placeholder(name: String) -> Result<String, minijinja::Error> {
    Ok(format!("{{{{ {name} }}}}"))
}

fn cond_vars(all_conds: &HashSet<String>, conds: &HashSet<String>) -> HashMap<String, bool> {
    let mut vars = HashMap::new();
    for c in all_conds {
        vars.insert(format!("cond__{c}"), conds.contains(c));
    }
    vars
}

fn capture_udvars<'a>(line: &'a str, re: &Regex, valid_udvars: &HashSet<String>) -> Vec<&'a str> {
    let mut result = vec![];
    for cap in re.captures_iter(line) {
        if let Some(g) = cap.get(1) {
            let var = g.as_str();
            if valid_udvars.contains(var) {
                result.push(g.as_str());
            }
        }
    }
    result
}

pub fn pos_args_mapping(template: &str, udvars: &HashSet<String>) -> HashMap<String, String> {
    let mut result: HashMap<String, u8> = HashMap::with_capacity(udvars.len());
    let re = Regex::new(r"\{\{\s?(\w+)\s?\}\}").unwrap();
    let mut counter = 1;
    for line in template.lines() {
        if line.is_empty() {
            continue;
        }
        for var in capture_udvars(line, &re, udvars) {
            if result.get(var).is_none() {
                result.insert(var.to_owned(), counter);
                counter += 1;
            }
        }
    }
    result
        .into_iter()
        .map(|(k, v)| (k, format!("${v}")))
        .collect::<HashMap<String, String>>()
}

pub fn variables_mapping(udvars: &HashSet<String>) -> HashMap<String, String> {
    udvars
        .iter()
        .map(|v| (v.to_owned(), format!(":{v}")))
        .collect::<HashMap<String, String>>()
}

fn strip_trailing_semicolon(s: &str) -> &str {
    s.strip_suffix(';').unwrap_or(s)
}

pub struct Engine<'a> {
    metadata: &'a Metadata,
    query_templates_env: Environment<'static>,
    test_templates_env: Environment<'static>,
}

impl<'a> From<&'a Metadata> for Engine<'a> {
    fn from(metadata: &'a Metadata) -> Self {
        // Env for query_templates
        let mut qt_env = Environment::new();
        qt_env.set_loader(path_loader(&metadata.query_templates_dir));
        qt_env.add_function("placeholder", placeholder);

        // Env for test_templates
        let mut tt_env = Environment::new();
        tt_env.set_loader(path_loader(&metadata.test_templates_dir));

        Self {
            metadata,
            query_templates_env: qt_env,
            test_templates_env: tt_env,
        }
    }
}

impl<'a> Engine<'a> {
    pub fn render_query(
        &self,
        query_id: &str,
        placeholder: Option<&Placeholder>,
    ) -> Result<String, Error> {
        let query = self
            .metadata
            .queries
            .get(query_id)
            .ok_or(Error::UndefinedQuery(query_id.to_owned()))?;
        let query_template = self.metadata.query_templates.get(&query.template).ok_or(
            Error::UndefinedQueryTemplate(query.template_file_name().to_owned()),
        )?;
        let tmpl = self
            .query_templates_env
            .get_template(query_template.file_name())
            .map_err(Error::MiniJinja)?;
        let ctx = cond_vars(&query_template.all_conds, &query.conds);
        let intermediate_output = tmpl.render(ctx).map_err(Error::MiniJinja)?;
        // Temporary environment to treat intermediate output as a
        // jinja template and render it
        let tmp_env = Environment::new();
        let intermediate_tmpl = tmp_env
            .template_from_str(&intermediate_output)
            .map_err(Error::MiniJinja)?;
        let udvars = intermediate_tmpl.undeclared_variables(false);
        let placeholder = placeholder.unwrap_or(&self.metadata.placeholder);
        let vars = match placeholder {
            Placeholder::PosArgs => pos_args_mapping(&intermediate_output, &udvars),
            Placeholder::Variables => variables_mapping(&udvars),
        };
        intermediate_tmpl.render(vars).map_err(Error::MiniJinja)
    }

    pub fn render_test(
        &self,
        path: &Path,
        prepared_statement: Option<&str>,
    ) -> Result<String, Error> {
        let test_template =
            self.metadata
                .test_templates
                .get(path)
                .ok_or(Error::UndefinedTestTemplate(
                    path.to_str().unwrap().to_owned(),
                ))?;
        let tmpl = self
            .test_templates_env
            .get_template(test_template.file_name())
            .map_err(Error::MiniJinja)?;
        // @TODO: Can we avoid allocation below by using `Cow`?
        let ps = match prepared_statement {
            Some(s) => s.to_owned(),
            None => self.render_query(&test_template.query, Some(&Placeholder::PosArgs))?,
        };
        let ctx = context! { prepared_statement => strip_trailing_semicolon(&ps) };
        tmpl.render(ctx).map_err(Error::MiniJinja)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn strset(xs: Vec<&str>) -> HashSet<String> {
        xs.iter().map(|s| String::from(*s)).collect()
    }

    #[test]
    fn test_cond_vars() {
        let all_conds = strset(vec!["a", "b", "c"]);
        let conds = strset(vec!["b", "c"]);
        let res = cond_vars(&all_conds, &conds);
        assert_eq!(3, res.len());
        assert_eq!(false, res["cond__a"]);
        assert_eq!(true, res["cond__b"]);
        assert_eq!(true, res["cond__c"]);
    }

    #[test]
    fn test_pos_args_mapping() {
        let udvars = HashSet::from_iter(vec![
            "firstname".to_owned(),
            "lastname".to_owned(),
            "department".to_owned(),
        ]);

        let template = r#"
SELECT
{{ firstname }}
{{ lastname }}
FROM employees
WHERE department = {{ department }} AND lastname = {{ lastname }}
AND tag = "{{ sometag }}"
;
"#;
        let result = pos_args_mapping(template, &udvars);
        assert_eq!(3, result.len());
        assert_eq!("$1", result.get("firstname").unwrap());
        assert_eq!("$2", result.get("lastname").unwrap());
        assert_eq!("$3", result.get("department").unwrap());

        let template = "";
        let result = pos_args_mapping(template, &udvars);
        assert_eq!(0, result.len());

        let template = "SELECT * from employees WHERE firstname = {{ firstname }};";
        let result = pos_args_mapping(template, &udvars);
        assert_eq!(1, result.len());
        assert_eq!("$1", result.get("firstname").unwrap());
    }
}
