use minijinja::context;

fn main() {
    let mut tmpl_env = minijinja::Environment::new();
    tmpl_env.set_loader(minijinja::path_loader("/Users/vineet/code/dastanu/db/sql_templates"));
    let tmpl = tmpl_env.get_template("q_shoplist.sql.j2").unwrap();
    let ctx = context! { cond__catid => true };
    let output = tmpl.render(ctx).unwrap();
    println!("{output:}");
}
