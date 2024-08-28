use super::config::Configurable;
use super::external::ExternalFormatter;
use crate::error::{parse_error, Error};
use crate::toml::{decode_pathbuf, SerializableTomlTable};
use std::path::{Path, PathBuf};
use toml::Value;

#[derive(Debug)]
pub struct SqlFluff {
    exec_path: PathBuf,
}

impl TryFrom<&Value> for SqlFluff {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_table() {
            Some(t) => {
                let exec_path = t
                    .get("exec_path")
                    .ok_or(parse_error!("Missing 'exec_path' in 'formatter.sqlfluff"))
                    .map(|v| decode_pathbuf(v, None, "formatter.sqlfluff.exec_path"))??;
                Ok(Self { exec_path })
            }
            None => Err(parse_error!(
                "Value of 'formatter.sqlfluff' must be a toml table"
            )),
        }
    }
}

impl Configurable for SqlFluff {
    fn to_toml_table(&self) -> SerializableTomlTable {
        let mut t = SerializableTomlTable::new("formatter.sqlfluff");
        t.push_comment("(required) Location of the sqlfluff executable");
        let exec_path = self.exec_path.display().to_string();
        t.push_entry_string("exec_path", &exec_path);
        t
    }

    fn config_file(&self) -> Option<(&Path, &'static str)> {
        // @TODO: Implement code to create the `.sqlfluff` config file
        None
    }
}

impl ExternalFormatter<'_> for SqlFluff {
    fn executable(&self) -> &Path {
        self.exec_path.as_path()
    }

    fn format_args(&self) -> Vec<&str> {
        vec!["format", "--dialect", "postgres", "-"]
    }

    fn check_args(&self) -> Vec<&str> {
        vec!["--version"]
    }
}

impl SqlFluff {
    pub fn discover() -> Option<Self> {
        let f = Self {
            exec_path: PathBuf::from("sqlfluff"),
        };
        if f.check() {
            Some(f)
        } else {
            None
        }
    }
}
