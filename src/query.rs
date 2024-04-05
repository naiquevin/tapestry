use crate::error::Error;
use crate::toml::{decode_string, decode_pathbuf, decode_vecstr};
use std::path::{Path, PathBuf};
use toml::Value;

#[allow(unused)]
#[derive(Debug)]
pub struct Query {
    id: String,
    template: PathBuf,
    conds: Vec<String>,
    output: Option<PathBuf>,
}

impl Query {

    pub fn decode<P: AsRef<Path>>(
        templates_base_dir: P,
        output_base_dir: P,
        value: &Value
    ) -> Result<Self, Error> {
        match value.as_table() {
            Some(t) => {
                let id = t.get("id")
                    .ok_or(Error::Parsing("Missing 'id' in 'query' entry".to_owned()))
                    .map(decode_string)??;
                let template = t.get("template")
                    .ok_or(Error::Parsing("Missing 'template' in 'query' entry".to_owned()))
                    .map(|v| decode_pathbuf(v, Some(templates_base_dir.as_ref())))??;
                let conds = t.get("conds")
                    .ok_or(Error::Parsing("Missing 'conds' in 'query' entry".to_owned()))
                    .map(decode_vecstr)??;
                let output = match t.get("option") {
                    Some(v) => Some(decode_pathbuf(v, Some(output_base_dir.as_ref()))?),
                    None => None
                };
                Ok(Self { id, template, conds, output })
            },
            None => Err(Error::Parsing("Invalid 'query' entry".to_owned()))
        }
    }
}

pub fn decode_queries<P: AsRef<Path>>(
    templates_base_dir: P,
    output_base_dir: P,
    value: &Value
) -> Result<Vec<Query>, Error> {
    match value.as_array() {
        Some(xs) => {
            let mut res = Vec::with_capacity(xs.len());
            for x in xs {
                let q = Query::decode(&templates_base_dir, &output_base_dir, x)?;
                res.push(q);
            }
            Ok(res)
        }
        None => Err(Error::Parsing("Invalid queries".to_owned()))
    }
}
