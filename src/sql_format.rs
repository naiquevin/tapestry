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
    args: Vec<&'a str>
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
            stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
        });

        let output = child.wait_with_output().expect("Failed to read stdout");
        output.stdout
    }
}

/// Provides an abstraction for formatting sql using the `pgFormatter`
/// (a.k.a `pg_format`) tool
#[derive(Debug)]
pub struct PgFormatter {
    exec_path: PathBuf,
    conf_path: Option<PathBuf>,
}

impl TryFrom<&Value> for PgFormatter {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_table() {
            Some(t) => {
                let exec_path = t.get("exec_path")
                    .ok_or(parse_error!("Missing 'exec_path' in 'formatter.pgFormatter"))
                    .map(|v| {
                        decode_pathbuf(v, None, "formatter.pgFormatter.exec_path")
                    })??;
                let conf_path = match t.get("conf_path") {
                    Some(cp) => Some(decode_pathbuf(cp, None, "formatter.pgFormatter.conf_path")?),
                    None => None
                };
                Ok(Self { exec_path, conf_path })
            },
            None => Err(parse_error!("Value of 'formatter.pgFormatter' must be a toml table"))
        }
    }
}

impl PgFormatter {

    pub fn format(&self, sql: &str) -> Vec<u8> {
        let mut default_args = vec!["-M", "-p", "start\\(noformat\\).+end\\(noformat\\)", "-"];
        let mut args = Vec::new();
        if let Some(conf) = &self.conf_path {
            args.push("-c");
            args.push(conf.to_str().unwrap())
        }
        args.append(&mut default_args);
        let cmd = Cmd {
            exec: &self.exec_path,
            args
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
}
