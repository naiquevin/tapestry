use minijinja::{context, Error};

fn placeholder(name: String) -> Result<String, Error> {
    Ok(format!("{{{{ {name} }}}}"))
}

fn main() {
    let mut tmpl_env = minijinja::Environment::new();
    tmpl_env.set_loader(minijinja::path_loader("examples"));
    tmpl_env.add_function("placeholder", placeholder);

    let tmpl = tmpl_env.get_template("songs.sql.j2").unwrap();
    let ctx = context! { cond__artist => true, cond__file_format => true };
    let output = tmpl.render(ctx).unwrap();
    println!("-- Intermediate query");
    println!("{output}");
    println!("");

    // The generated output is again a jinja2 template. We add it to
    // the env and find the undeclared_variables
    tmpl_env.add_template("_gen1", &output).unwrap();
    let gen_tmpl = tmpl_env.get_template("_gen1").unwrap();
    let udvars = gen_tmpl.undeclared_variables(false);
    println!("-- {udvars:?}");
    println!("");

    // @NOTE that the query for production will *NOT* be constructed
    // as follows. Instead the intermediate query will be parsed and
    // the using with the `undeclared_variables`, the placeholders in
    // the intermediate query will be mapped to the positional
    // arguments automatically.
    let query = gen_tmpl.render(context! {
        artist => "$1",
        file_format => "$2"
    }).unwrap();
    println!("-- Query for production code");
    println!("{query}");
    println!("");

    let test_query = gen_tmpl.render(context! {
        artist => "'Iron Maiden'",
        file_format => "'Protected AAC audio file'"
    }).unwrap();
    println!("-- Query for test code");
    println!("{test_query}");
    println!("");
}
