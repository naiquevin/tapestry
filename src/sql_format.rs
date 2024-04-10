use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

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

pub struct PgFormatter {
    exec_path: PathBuf,
}

impl PgFormatter {

    pub fn new(exec: &str) -> Self {
        Self { exec_path: PathBuf::from(exec) }
    }

    pub fn format(&self, sql: &str) -> Vec<u8> {
        let cmd = Cmd {
            exec: &self.exec_path,
            args: vec!["-M", "-p", "start\\(noformat\\).+end\\(noformat\\)", "-"],
        };
        cmd.execute(sql)
    }
}
