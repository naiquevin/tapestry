use crate::metadata::MetaData;
use crate::placeholder::Placeholder;
use crate::render::{placeholder, pos_args_mapping, variables_mapping, Engine};
use minijinja::context;

mod error;
mod metadata;
mod placeholder;
mod query;
mod query_template;
mod render;
mod test_template;
mod toml;
mod validation;

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

fn main() {
    let path = std::path::Path::new("manifest.toml");
    match MetaData::try_from(path) {
        Ok(m) => {
            // println!("{m:?}")
            let mistakes = m.validate();
            if mistakes.is_empty() {
                let engine = Engine::from(&m);
                let qid = "artists_long_songs@genre*limit";
                let query_output = engine.render_query(qid, None).unwrap();
                println!("{query_output}");
                println!("---------------");
                let ps = match m.placeholder {
                    Placeholder::PosArgs => Some(query_output.as_str()),
                    Placeholder::Variables => None,
                };
                for tt in m.test_templates.find_by_query(qid) {
                    let test_output = engine.render_test(&tt.path, ps).unwrap();
                    println!("{test_output}");
                    println!("---------------");
                }
            } else {
                println!("{mistakes:?}");
            }
        }
        Err(e) => println!("{e:?}"),
    }
}
