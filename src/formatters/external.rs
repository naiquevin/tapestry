use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

use toml::Value;

/// Trait for formatters that are installed as external programs.
#[allow(unused)]
pub trait ExternalFormatter<'a>: TryFrom<&'a Value> {
    /// Returns path to the executable
    fn executable(&self) -> &Path;

    /// Returns a vector of arguments to be specified when calling the
    /// program for formatting
    fn format_args(&self) -> Vec<&str>;

    /// Returns a vector of arguments that the program can be called
    /// with to check that the program is installed
    fn check_args(&self) -> Vec<&str>;

    /// Returns a byte array of formatted input.
    ///
    /// The default implementation calls shell's out to the executable
    /// with by passing the input string through `stdin` along with
    /// the result of `self.format_args` as arguments.
    fn format(&self, input: &str) -> Vec<u8> {
        let mut child = Command::new(self.executable())
            .args(&self.format_args())
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

    /// Checks whether or not the executable exists (i.e. the
    /// external formatting program is installed)
    ///
    /// The default implementation tries running the executable with
    /// the return value of `self.check_args` as arguments.
    fn check(&self) -> bool {
        Command::new(self.executable())
            .args(&self.check_args())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("Failed to spawn child process")
            .success()
    }
}
