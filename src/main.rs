use crate::error::Error;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

mod command;
mod error;
mod metadata;
mod output;
mod placeholder;
mod query;
mod query_template;
mod render;
mod scaffolding;
mod sql_format;
mod test_template;
mod toml;
mod validation;

#[derive(Subcommand)]
enum Command {
    Init { path: PathBuf },
    Validate,
    Render,
    Summary,
    Status {
        #[arg(
            long,
            default_value_t = false,
            help = "Exit with non-zero code if any templates have unrendered changes"
        )]
        assert_no_changes: bool,
    },
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

impl Cli {
    fn execute(&self) -> Result<i32, Error> {
        match &self.command {
            Some(Command::Init { path }) => command::init(path),
            Some(Command::Validate) => command::validate(),
            Some(Command::Render) => command::render(),
            Some(Command::Summary) => command::summary(),
            Some(Command::Status { assert_no_changes }) => command::status(*assert_no_changes),
            None => Err(Error::Cli("Please specify the command".to_owned())),
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let result = cli.execute();
    match result {
        Ok(status) => process::exit(status),
        Err(Error::Cli(msg)) => {
            eprintln!("Command error: {}", msg);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Error {:?}", e);
            process::exit(1)
        }
    }
}
