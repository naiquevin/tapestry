use minijinja::{context, Error};
use regex::Regex;
use std::collections::{HashMap, HashSet};

fn placeholder(name: String) -> Result<String, Error> {
    Ok(format!("{{{{ {name} }}}}"))
}

// #[allow(dead_code)]
// fn udvars_to_regex(udvars: &HashSet<String>) -> Result<Regex, regex::Error> {
//     let segs = udvars.iter()
//         .map(|var| {
//             format!("{{{{\\s?({var})\\s?}}}}")
//         })
//         .collect::<Vec<String>>();
//     let pat_str = segs.join("|");
//     let pat_estr = regex::escape(&pat_str);
//     Regex::new(&pat_estr)
// }

fn capture_udvars<'a>(line: &'a str, re: &Regex, valid_udvars: &HashSet<String>) -> Vec<&'a str> {
    let mut result = vec![];
    for cap in re.captures_iter(&line) {
        if let Some(g) = cap.get(1) {
            let var = g.as_str();
            if valid_udvars.contains(var) {
                result.push(g.as_str());
            }
        }
    }
    result
}

fn pos_args_mapping(template: &str, udvars: &HashSet<String>) -> HashMap<String, String> {
    let mut result: HashMap<String, u8> = HashMap::with_capacity(udvars.len());
    let re = Regex::new(r"\{\{\s?(\w+)\s?\}\}").unwrap();
    let mut counter = 1;
    for line in template.lines() {
        if line.is_empty() {
            continue;
        }
        for var in capture_udvars(&line, &re, udvars) {
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

fn variables_mapping(udvars: &HashSet<String>) -> HashMap<String, String> {
    udvars
        .iter()
        .map(|v| (v.to_owned(), format!(":{v}")))
        .collect::<HashMap<String, String>>()
}

fn main() {
    let mut tmpl_env = minijinja::Environment::new();
    tmpl_env.set_loader(minijinja::path_loader("examples/sql"));
    tmpl_env.add_function("placeholder", placeholder);

    let tmpl = tmpl_env.get_template("songs.sql.j2").unwrap();
    let ctx = context! { cond__artist => true, cond__file_format => true };
    let output = tmpl.render(ctx).unwrap();
    println!("-- Intermediate query");
    println!("{output}");
    println!("");

    // The generated output is again a jinja2 template. We add it to
    // the env and find the undeclared_variables
    tmpl_env.add_template("_gen1", &output).unwrap();
    let gen_tmpl = tmpl_env.get_template("_gen1").unwrap();
    let udvars = gen_tmpl.undeclared_variables(false);

    println!("-- Query with positional arguments (for production code)");
    let pos_args = pos_args_mapping(&output, &udvars);
    println!("-- Positional args: {pos_args:?}");
    let query = gen_tmpl.render(&pos_args).unwrap();
    println!("{query}");
    println!("");

    println!("-- Query with variables mapping (for production code)");
    let vars_map = variables_mapping(&udvars);
    println!("-- Query variables: {vars_map:?}");
    let query = gen_tmpl.render(&vars_map).unwrap();
    println!("{query}");
    println!("");

    let test_query = gen_tmpl
        .render(context! {
            artist => "'Iron Maiden'",
            file_format => "'Protected AAC audio file'"
        })
        .unwrap();
    println!("-- Query for test code");
    println!("{test_query}");
    println!("");
}

#[cfg(test)]
mod tests {

    use super::*;

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
