use crate::error::{parse_error, Error};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use toml::Value;

/// Tries decoding a toml `Value` into a `String`
///
/// The second arg `key` will be used in the error message in case
/// decoding fails (i.e. in case the value in the toml file is not a
/// string).
pub fn decode_string(value: &Value, key: &str) -> Result<String, Error> {
    value
        .as_str()
        .ok_or(parse_error!("Value of '{}' expected to be a string", key))
        .map(|s| s.to_owned())
}

/// Tries decoding a toml `Value` into a PathBuf
///
/// The second arg `key` will be used in the error message in
/// case decoding fails (i.e. in case the value in the toml file is
/// not a string).
pub fn decode_pathbuf(value: &Value, base_dir: Option<&Path>, key: &str) -> Result<PathBuf, Error> {
    value
        .as_str()
        .ok_or(parse_error!(
            "Value of '{}' is expected to be a string",
            key
        ))
        .map(|s| base_dir.map_or_else(|| PathBuf::from(s), |p| p.join(s)))
}

/// Tries decoding a toml `Value` into `HashSet<String>`
///
/// The second arg `key` will be used in the error message in
/// case decoding fails (i.e. in case the value in the toml file is
/// not an array of strings).
pub fn decode_strset(value: &Value, key: &str) -> Result<HashSet<String>, Error> {
    match value.as_array() {
        Some(xs) => {
            let mut res = HashSet::with_capacity(xs.len());
            for v in xs {
                match v.as_str() {
                    Some(x) => {
                        res.insert(x.to_owned());
                    },
                    None => {
                        return Err(parse_error!(
                            "Value of '{}' is expected to be array of strings",
                            key
                        ))
                    }
                }
            }
            Ok(res)
        }
        None => Err(parse_error!(
            "Value of '{}' is expected to be an array of strings",
            key
        )),
    }
}
