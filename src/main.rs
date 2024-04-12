use crate::error::Error;
use crate::metadata::Metadata;
use crate::placeholder::Placeholder;
use crate::render::{placeholder, pos_args_mapping, variables_mapping, Engine};
use clap::{Parser, Subcommand};
use minijinja::context;
use std::process;

mod command;
mod error;
mod metadata;
mod output;
mod placeholder;
mod query;
mod query_template;
mod render;
mod sql_format;
mod test_template;
mod toml;
mod validation;

#[derive(Subcommand)]
enum Command {
    Validate,
    Render,
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

impl Cli {
    fn execute(&self) -> Result<(), Error> {
        match &self.command {
            Some(Command::Validate) => command::validate(),
            Some(Command::Render) => command::render(),
            None => Err(Error::Cli("Please specify the command".to_owned()))
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let result = cli.execute();
    match result {
        Ok(()) => process::exit(0),
        Err(Error::Cli(msg)) => {
            eprintln!("Command error: {}", msg);
            process::exit(1);
        },
        Err(Error::InvalidManifest) => process::exit(1),
        Err(e) => {
            eprintln!("Error {:?}", e);
            process::exit(1)
        }
    }
}


#[allow(dead_code)]
fn main2() {
    let mut tmpl_env = minijinja::Environment::new();
    tmpl_env.set_loader(minijinja::path_loader("templates/queries"));
    tmpl_env.add_function("placeholder", placeholder);

    let tmpl = tmpl_env.get_template("artists_long_songs.sql.j2").unwrap();
    let ctx = context! { cond__genre => true, cond__limit => true };
    let output = tmpl.render(ctx).unwrap();
    println!("-- Intermediate query");
    println!("{output}");
    println!();

    // The generated output is again a jinja2 template. We add it to
    // the env and find the undeclared_variables
    tmpl_env.add_template("_gen1", &output).unwrap();
    let gen_tmpl = tmpl_env.get_template("_gen1").unwrap();
    let udvars = gen_tmpl.undeclared_variables(false);

    println!("-- Query with positional arguments (for production code)");
    let pos_args = pos_args_mapping(&output, &udvars);
    println!("-- Positional args: {pos_args:?}");
    let query = gen_tmpl.render(&pos_args).unwrap();
    println!("{query}");
    println!();

    println!("-- Query with variables mapping (for production code)");
    let vars_map = variables_mapping(&udvars);
    println!("-- Query variables: {vars_map:?}");
    let query = gen_tmpl.render(&vars_map).unwrap();
    println!("{query}");
    println!();

    let test_query = gen_tmpl
        .render(context! {
            genre => "'Rock'",
            limit => "10"
        })
        .unwrap();
    println!("-- Query for test code");
    println!("{test_query}");
    println!();
}

#[allow(dead_code)]
fn main3() {
    let path = std::path::Path::new("tapestry.toml");
    match Metadata::try_from(path) {
        Ok(m) => {
            // println!("{m:?}")
            let mistakes = m.validate();
            if mistakes.is_empty() {
                let engine = Engine::from(&m);
                let qid = "artists_long_songs@genre*limit";
                let query_output = engine.render_query(qid, None).unwrap();
                if let Some(formatter) = &m.formatter {
                    let fop = formatter.format(&query_output);
                    println!("{}", String::from_utf8_lossy(&fop));
                } else {
                    println!("{query_output}");
                }
                println!("---------------");
                let ps = match m.placeholder {
                    Placeholder::PosArgs => Some(query_output.as_str()),
                    Placeholder::Variables => None,
                };
                for tt in m.test_templates.find_by_query(qid) {
                    let test_output = engine.render_test(&tt.path, ps).unwrap();
                    if let Some(formatter) = &m.formatter {
                        let ftop = formatter.format(&test_output);
                        println!("{}", String::from_utf8_lossy(&ftop));
                    } else {
                        println!("{test_output}");
                    }
                    println!("---------------");
                }
            } else {
                for mistake in mistakes {
                    println!("{}", mistake.err_msg())
                }
            }
        }
        Err(e) => println!("{e:?}"),
    }
}
