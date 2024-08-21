use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

/// Provides an abstraction for running an executable (i.e. shelling
/// out) to format sql. It passes raw unformatted sql as input to the
/// executable via `stdin`.
pub struct Cmd<'a> {
    pub exec: &'a Path,
    pub args: Vec<&'a str>,
}

impl<'a> Cmd<'a> {
    pub fn execute(&self, input: &str) -> Vec<u8> {
        let mut child = Command::new(self.exec)
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
