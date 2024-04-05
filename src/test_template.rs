use crate::error::Error;
use crate::toml::{decode_string, decode_pathbuf};
use std::path::{Path, PathBuf};
use toml::Value;

#[allow(unused)]
#[derive(Debug)]
pub struct TestTemplate {
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
                let query = t.get("query")
                    .ok_or(Error::Parsing("Missing 'query' in 'test_templates' entry".to_owned()))
                    .map(decode_string)??;
                let template = t.get("template")
                    .ok_or(Error::Parsing("Missing 'template' in 'test_templates' entry".to_owned()))
                    .map(|v| decode_pathbuf(v, Some(templates_base_dir.as_ref())))??;
                let output = match t.get("output") {
                    Some(v) => Some(decode_pathbuf(v, Some(output_base_dir.as_ref()))?),
                    None => None,
                };
                Ok(Self { query, template, output })
            },
            None => Err(Error::Parsing("Invalid 'test_templates' entry".to_owned()))
        }
    }
}

pub fn decode_test_templates<P: AsRef<Path>>(
    templates_base_dir: P,
    output_base_dir: P,
    value: &Value
) -> Result<Vec<TestTemplate>, Error> {
    match value.as_array() {
        Some(xs) => {
            let mut res = Vec::with_capacity(xs.len());
            for x in xs {
                let tt = TestTemplate::decode(&templates_base_dir, &output_base_dir, x)?;
                res.push(tt);
            }
            Ok(res)
        },
        None => Err(Error::Parsing("Invalid test_templates".to_owned()))
    }
}
