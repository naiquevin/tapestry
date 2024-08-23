use crate::error::{parse_error, Error};
use crate::toml::decode_pathbuf;
use std::cell::OnceCell;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use toml::Value;

use super::util::Cmd;

/// Provides an abstraction for formatting sql using the `pgFormatter`
/// (a.k.a `pg_format`) tool
#[derive(Debug)]
pub struct PgFormatter {
    pub exec_path: PathBuf,
    pub conf_path: Option<PathBuf>,
    args: OnceCell<Vec<String>>,
}

fn pg_format_args(conf_path: Option<&Path>) -> Vec<String> {
    let args = match conf_path {
        Some(conf) => vec!["-c", conf.to_str().unwrap(), "-"],
        None => vec!["-M", "-p", "start\\(noformat\\).+end\\(noformat\\)", "-"],
    };
    args.into_iter().map(String::from).collect()
}

impl TryFrom<&Value> for PgFormatter {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_table() {
            Some(t) => {
                let exec_path = t
                    .get("exec_path")
                    .ok_or(parse_error!(
                        "Missing 'exec_path' in 'formatter.pgFormatter"
                    ))
                    .map(|v| decode_pathbuf(v, None, "formatter.pgFormatter.exec_path"))??;
                let conf_path = match t.get("conf_path") {
                    Some(cp) => Some(decode_pathbuf(cp, None, "formatter.pgFormatter.conf_path")?),
                    None => None,
                };
                Ok(Self::new(exec_path, conf_path))
            }
            None => Err(parse_error!(
                "Value of 'formatter.pgFormatter' must be a toml table"
            )),
        }
    }
}

impl PgFormatter {

    pub fn new(exec_path: PathBuf, conf_path: Option<PathBuf>) -> Self {
        Self {
            exec_path,
            conf_path,
            args: OnceCell::new(),
        }
    }

    pub fn new_if_exists() -> Option<Self> {
        let mut command = Command::new("pg_format");
        let status = command
            .arg("-v")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("Failed to spawn child process");
        if status.success() {
            Some(Self::new(
                PathBuf::from(command.get_program()),
                Some(PathBuf::from("./.pg_format/config")))
            )
        } else {
            None
        }
    }

    pub fn format(&self, sql: &str) -> Vec<u8> {
        let args = self.args
            .get_or_init(|| pg_format_args(self.conf_path.as_deref()))
            .iter()
            .map(|a| a.as_str()).collect();
        let cmd = Cmd {
            exec: &self.exec_path,
            args,
        };
        cmd.execute(sql)
    }
}

#[cfg(test)]
mod tests {

    use super::pg_format_args;
    use std::path::Path;

    #[test]
    fn test_pg_format_args() {
        let args = pg_format_args(None);
        let expected = vec!["-M", "-p", "start\\(noformat\\).+end\\(noformat\\)", "-"];
        assert_eq!(
            expected
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>(),
            args
        );

        let conf_path = Path::new("./.pg_format/config");
        let args = pg_format_args(Some(&conf_path));
        let expected = vec!["-c", "./.pg_format/config", "-"];
        assert_eq!(
            expected
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>(),
            args
        );
    }
}
