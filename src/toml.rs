use crate::error::{Error, parse_error};
use std::path::{Path, PathBuf};
use toml::Value;

pub fn decode_string(value: &Value) -> Result<String, Error> {
    value.as_str()
        .ok_or(parse_error!("Value expected to be a string"))
        .map(|s| s.to_owned())
}

pub fn decode_pathbuf(value: &Value, base_dir: Option<&Path>) -> Result<PathBuf, Error> {
    value.as_str()
        .ok_or(parse_error!("Value expected to be a string"))
        .map(|s| base_dir.map_or_else(|| PathBuf::from(s), |p| p.join(s)))
}

pub fn decode_vecstr(value: &Value) -> Result<Vec<String>, Error> {
    match value.as_array() {
        Some(xs) => {
            let mut res = Vec::with_capacity(xs.len());
            for v in xs {
                match v.as_str() {
                    Some(x) => res.push(x.to_owned()),
                    None => return Err(parse_error!("Value expected to be a string"))
                }
            }
            Ok(res)
        }
        None => Err(parse_error!("Value expected to be an array"))
    }
}
