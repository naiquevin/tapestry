use crate::error::{parse_error, Error};
use crate::toml::{decode_pathbuf, decode_string};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use toml::Value;

#[allow(unused)]
#[derive(Debug)]
struct TestTemplate {
    query: String,
    template: PathBuf,
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
}

#[allow(unused)]
#[derive(Debug)]
pub struct TestTemplates {
    inner: Vec<Rc<TestTemplate>>,
    cache: HashMap<String, Rc<TestTemplate>>,
}

impl TestTemplates {
    pub fn new() -> Self {
        let inner: Vec<Rc<TestTemplate>> = vec![];
        let cache: HashMap<String, Rc<TestTemplate>> = HashMap::new();
        Self { inner, cache }
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
        let cache: HashMap<String, Rc<TestTemplate>> = HashMap::new();
        Ok(Self {
            inner: items,
            cache,
        })
    }
}
