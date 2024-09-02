use crate::error::{parse_error, Error};
use crate::toml::{decode_pathbuf, SerializableTomlTable};
use std::cell::OnceCell;
use std::path::{Path, PathBuf};
use toml::Value;

use super::config::Configurable;
use super::external::ExternalFormatter;

#[derive(Debug)]
pub struct SqlFormatter {
    exec_path: PathBuf,
    conf_path: Option<PathBuf>,
    args: OnceCell<Vec<String>>,
}

impl TryFrom<&Value> for SqlFormatter {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_table() {
            Some(t) => {
                let exec_path = t
                    .get("exec_path")
                    .ok_or(parse_error!(
                        "Missing 'exec_path' in 'formatter.sql-formatter"
                    ))
                    .map(|v| decode_pathbuf(v, None, "formatter.sql-formatter.exec_path"))??;
                let conf_path = match t.get("conf_path") {
                    Some(cp) => Some(decode_pathbuf(
                        cp,
                        None,
                        "formatter.sql-formatter.conf_path",
                    )?),
                    None => None,
                };
                Ok(Self::new(exec_path, conf_path))
            }
            None => Err(parse_error!(
                "Value of 'formatter.sql-formatter' must be a toml table"
            )),
        }
    }
}

impl SqlFormatter {
    pub fn new(exec_path: PathBuf, conf_path: Option<PathBuf>) -> Self {
        Self {
            exec_path,
            conf_path,
            args: OnceCell::new(),
        }
    }

    pub fn discover() -> Option<Self> {
        let f = Self::new(
            PathBuf::from("sql-formatter"),
            Some(PathBuf::from("./.sql-formatter/config.json")),
        );
        if f.check() {
            Some(f)
        } else {
            None
        }
    }
}

impl Configurable for SqlFormatter {
    fn to_toml_table(&self) -> SerializableTomlTable {
        let mut t = SerializableTomlTable::new("formatter.sql-formatter");
        t.push_comment("(required) Location of the sql-formatter executable");
        let exec_path = &self.exec_path.display().to_string();
        t.push_entry_string("exec_path", exec_path);
        t.push_comment("(optional) path to the json conf file.");
        if let Some(p) = &self.conf_path {
            let conf_path = &p.display().to_string();
            t.push_entry_string("conf_path", conf_path);
        }
        t
    }

    fn config_file(&self) -> Option<(&Path, &'static str)> {
        self.conf_path
            .as_deref()
            .map(|p| (p, include_str!("../../defaults/sql-formatter.config.json")))
    }
}

fn sql_formatter_args(conf_path: Option<&Path>) -> Vec<String> {
    let args = match conf_path {
        Some(conf) => vec!["-c", conf.to_str().unwrap()],
        None => vec![],
    };
    args.into_iter().map(String::from).collect()
}

impl ExternalFormatter<'_> for SqlFormatter {
    fn executable(&self) -> &Path {
        self.exec_path.as_path()
    }

    fn format_args(&self) -> Vec<&str> {
        let args = self
            .args
            .get_or_init(|| sql_formatter_args(self.conf_path.as_deref()));
        args.iter().map(|a| a.as_str()).collect()
    }

    fn check_args(&self) -> Vec<&str> {
        vec!["--version"]
    }
}
