use crate::error::{parse_error, Error};
use crate::toml::decode_pathbuf;
use std::convert::TryFrom;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use toml::Value;

/// Provides an abstraction for running an executable (i.e. shelling
/// out) to format sql. It passes raw unformatted sql as input to the
/// executable via `stdin`.
struct Cmd<'a> {
    exec: &'a Path,
    args: Vec<&'a str>,
}

impl<'a> Cmd<'a> {
    fn execute(&self, input: &str) -> Vec<u8> {
        let mut child = Command::new(&self.exec)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn child process");

        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        // @TODO: Check if it's possible to avoid allocating for an
        // owned String here
        let input = input.to_owned();
        thread::spawn(move || {
            stdin
                .write_all(input.as_bytes())
                .expect("Failed to write to stdin");
        });

        let output = child.wait_with_output().expect("Failed to read stdout");
        output.stdout
    }
}

/// Provides an abstraction for formatting sql using the `pgFormatter`
/// (a.k.a `pg_format`) tool
#[derive(Debug)]
pub struct PgFormatter {
    pub exec_path: PathBuf,
    pub conf_path: Option<PathBuf>,
    args: Vec<String>,
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
                let args = pg_format_args(conf_path.as_ref().map(|p| p.as_path()));
                Ok(Self {
                    exec_path,
                    conf_path,
                    args,
                })
            }
            None => Err(parse_error!(
                "Value of 'formatter.pgFormatter' must be a toml table"
            )),
        }
    }
}

impl PgFormatter {

    pub fn new_if_exists() -> Option<Self> {
        let mut command = Command::new("pg_format");
        let status = command
            .arg("-v")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("Failed to spawn child process");
        if status.success() {
            Some(Self {
                exec_path: PathBuf::from(command.get_program()),
                conf_path: Some(PathBuf::from("./.pg_format/config")),
                args: pg_format_args(None),
            })
        } else {
            None
        }
    }

    pub fn format(&self, sql: &str) -> Vec<u8> {
        let args = self.args.iter().map(|a| a.as_str()).collect();
        let cmd = Cmd {
            exec: &self.exec_path,
            args,
        };
        cmd.execute(sql)
    }
}

/// Enum wrapping over abstractions for various sql formatting tools.
///
/// This indirection is just a provision for plugging in sql
/// formatting tools other than PgFormatter. But at present, only
/// `PgFormatter` is supported.
#[derive(Debug)]
pub enum Formatter {
    PgFormatter(PgFormatter),
}

impl Formatter {
    pub fn decode(value: &Value) -> Result<Option<Self>, Error> {
        match value.as_table() {
            Some(t) => {
                if let Some(v) = t.get("pgFormatter") {
                    PgFormatter::try_from(v).map(|f| Some(Self::PgFormatter(f)))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    pub fn format(&self, sql: &str) -> Vec<u8> {
        match self {
            Self::PgFormatter(p) => p.format(sql),
        }
    }

    pub fn discover() -> Option<Self> {
        let pg_format = PgFormatter::new_if_exists().map(Self::PgFormatter);
        if pg_format.is_some() {
            return pg_format
        } else {
            // Check for more formatting tools here when support for
            // them is added.
            None
        }
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
        assert_eq!(expected.into_iter().map(String::from).collect::<Vec<String>>(), args);

        let conf_path = Path::new("./.pg_format/config");
        let args = pg_format_args(Some(&conf_path));
        let expected = vec!["-c", "./.pg_format/config", "-"];
        assert_eq!(expected.into_iter().map(String::from).collect::<Vec<String>>(), args);
    }
}
