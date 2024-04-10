use crate::error::{parse_error, Error};
use crate::query::Queries;
use crate::toml::{decode_pathbuf, decode_string};
use crate::validation::{validate_path, ManifestMistake};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use toml::Value;

#[derive(Debug)]
pub struct TestTemplate {
    pub query: String,
    pub template: PathBuf,
    output: Option<PathBuf>,
}

impl TestTemplate {
    fn decode<P: AsRef<Path>>(
        templates_base_dir: P,
        output_base_dir: P,
        value: &Value,
    ) -> Result<Self, Error> {
        match value.as_table() {
            Some(t) => {
                let query = t
                    .get("query")
                    .ok_or(parse_error!("Missing 'query' in 'test_templates' entry"))
                    .map(|v| decode_string(v, "test_templates[].query"))??;
                let template = t
                    .get("template")
                    .ok_or(parse_error!("Missing 'template' in 'test_templates' entry"))
                    .map(|v| {
                        decode_pathbuf(
                            v,
                            Some(templates_base_dir.as_ref()),
                            "test_templates[].template",
                        )
                    })??;
                let output = match t.get("output") {
                    Some(v) => Some(decode_pathbuf(
                        v,
                        Some(output_base_dir.as_ref()),
                        "test_templates[].output",
                    )?),
                    None => None,
                };
                Ok(Self {
                    query,
                    template,
                    output,
                })
            }
            None => Err(parse_error!("Invalid 'test_templates' entry")),
        }
    }

    fn validate<'a, 'b>(&'a self, queries: &'b Queries) -> Vec<ManifestMistake<'a>>
    where
        'b: 'a,
    {
        let mut mistakes = vec![];
        if queries.get(&self.query).is_none() {
            mistakes.push(ManifestMistake::QueryRefNotFound {
                query_id: &self.query,
                test_template: self.template.to_str().unwrap(),
            });
        }

        match validate_path(&self.template, "test_templates[].template") {
            Ok(()) => {}
            Err(m) => mistakes.push(m),
        }
        mistakes
    }

    /// Returns file name of the template which can be used with
    /// `minijinja::Environment` that's initialized using
    /// `minijinja::path_loader`
    ///
    /// # Panics
    ///
    /// 1. This fn assumes that the template path is valid unicode and
    /// will panic if that's not the case.
    ///
    /// 2. If the path ends in `..`
    ///
    pub fn file_name(&self) -> &str {
        self.template
            .file_name()
            .map(|ostr| ostr.to_str().unwrap())
            .unwrap()
    }
}

#[derive(Debug)]
pub struct TestTemplates {
    inner: Vec<Rc<TestTemplate>>,
}

impl TestTemplates {
    pub fn new() -> Self {
        let inner: Vec<Rc<TestTemplate>> = vec![];
        Self { inner }
    }

    pub fn decode<P: AsRef<Path>>(
        templates_base_dir: P,
        output_base_dir: P,
        value: &Value,
    ) -> Result<Self, Error> {
        let items = match value.as_array() {
            Some(xs) => {
                let mut res = Vec::with_capacity(xs.len());
                for x in xs {
                    let tt = TestTemplate::decode(&templates_base_dir, &output_base_dir, x)?;
                    res.push(Rc::new(tt));
                }
                res
            }
            None => return Err(parse_error!("Invalid test_templates")),
        };
        Ok(Self { inner: items })
    }

    pub fn validate<'a, 'b>(&'a self, queries: &'b Queries) -> Vec<ManifestMistake>
    where
        'b: 'a,
    {
        let mut mistakes = vec![];
        let count = self.inner.len();
        let mut all_outputs: HashMap<&Path, usize> = HashMap::with_capacity(count);
        for tt in &self.inner {
            mistakes.append(&mut tt.validate(queries));
            if let Some(o) = &tt.output {
                all_outputs.entry(o).and_modify(|c| *c += 1).or_insert(1);
            }
        }
        for (key, val) in all_outputs.iter() {
            if val > &1 {
                let m = ManifestMistake::Duplicates {
                    key: "test_templates[].path",
                    value: key.to_str().unwrap(),
                };
                mistakes.push(m)
            }
        }
        mistakes
    }

    pub fn get(&self, template_path: &Path) -> Option<&Rc<TestTemplate>> {
        self.inner.iter().find(|tt| tt.template == template_path)
    }

    /// Returns all test templates for the given `query_id`
    ///
    /// Note that currently this fn scans through the inner data
    /// structure so it's not as performant. As all the various lookup
    /// patterns for test_templates become more clear, we will decide
    /// to either use an index or modify inner itself to use a
    /// suitable data structure such as a `HashMap` of Strings (query
    /// ids) mapping to `Vec<TestTemplate>`.
    pub fn find_by_query(&self, query_id: &str) -> Vec<&Rc<TestTemplate>> {
        self.inner
            .iter()
            .filter(|tt| tt.query.as_str() == query_id)
            .collect()
    }
}
