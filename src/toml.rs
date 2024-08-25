use crate::error::{parse_error, Error};
use serde::{Serialize, Serializer};
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
                    }
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

// Abstractions for serializing simple toml tables

enum SerializableTomlTableLine {
    Entry(String, Value),
    Comment(String),
}

impl Serialize for SerializableTomlTableLine {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            Self::Entry(k, v) => format!("{k} = {v}"),
            Self::Comment(msg) => format!("# {msg}"),
        };
        serializer.serialize_str(s.as_str())
    }
}

#[derive(Serialize)]
pub struct SerializableTomlTable {
    key_path: String,
    lines: Vec<SerializableTomlTableLine>,
}

impl SerializableTomlTable {
    pub fn new(key_path: &str) -> Self {
        Self {
            key_path: key_path.to_owned(),
            lines: vec![],
        }
    }

    pub fn push_entry_string(&mut self, key: &str, value: &str) {
        let v = Value::String(value.to_owned());
        let entry = SerializableTomlTableLine::Entry(key.to_owned(), v);
        self.lines.push(entry)
    }

    pub fn push_entry_i64(&mut self, key: &str, value: i64) {
        let v = Value::Integer(value);
        let entry = SerializableTomlTableLine::Entry(key.to_owned(), v);
        self.lines.push(entry)
    }

    pub fn push_entry_bool(&mut self, key: &str, value: bool) {
        let v = Value::Boolean(value);
        let entry = SerializableTomlTableLine::Entry(key.to_owned(), v);
        self.lines.push(entry)
    }

    pub fn push_comment(&mut self, msg: &str) {
        let comment = SerializableTomlTableLine::Comment(msg.to_owned());
        self.lines.push(comment)
    }
}
