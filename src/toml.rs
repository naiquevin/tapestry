use crate::error::Error;
use std::path::{Path, PathBuf};
use toml::Value;

pub fn decode_string(value: &Value) -> Result<String, Error> {
    value.as_str()
        .ok_or(Error::Parsing("Value expected to be a string".to_owned()))
        .map(|s| s.to_owned())
}

pub fn decode_pathbuf(value: &Value, base_dir: Option<&Path>) -> Result<PathBuf, Error> {
    value.as_str()
        .ok_or(Error::Parsing("Value expected to be a string".to_owned()))
        .map(|s| base_dir.map_or_else(|| PathBuf::from(s), |p| p.join(s)))
}

pub fn decode_vecstr(value: &Value) -> Result<Vec<String>, Error> {
    match value.as_array() {
        Some(xs) => {
            let mut res = Vec::with_capacity(xs.len());
            for v in xs {
                match v.as_str() {
                    Some(x) => res.push(x.to_owned()),
                    None => return Err(Error::Parsing("Value expected to be a string".to_owned()))
                }
            }
            Ok(res)
        }
        None => Err(Error::Parsing("Value expected to be an array".to_owned()))
    }
}
