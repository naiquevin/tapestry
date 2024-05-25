use crate::error::Error;
use crate::metadata::Metadata;
use crate::output;
use crate::placeholder::Placeholder;
use crate::render::Engine;
use crate::scaffolding;
use comfy_table::Table;
use std::collections::HashMap;
use std::path::Path;

pub fn validate() -> Result<i32, Error> {
    let path = Path::new("tapestry.toml");
    let metadata = Metadata::try_from(path)?;
    let mistakes = metadata.validate();
    if mistakes.is_empty() {
        println!("All Ok: Manifest file '{}' is valid", path.display());
        Ok(0)
    } else {
        println!("Invalid manifest file: '{}'", path.display());
        for mistake in mistakes {
            println!("{}", mistake.err_msg())
        }
        Ok(1)
    }
}

pub fn render() -> Result<i32, Error> {
    let path = Path::new("tapestry.toml");
    let metadata = Metadata::try_from(path)?;
    let mistakes = metadata.validate();
    if mistakes.is_empty() {
        let engine = Engine::from(&metadata);
        let formatter = &metadata.formatter;
        output::ensure_output_dirs(&metadata.queries_output_dir, &metadata.tests_output_dir)?;
        for query in metadata.queries.iter() {
            // render and process query
            let query_output = engine.render_query(&query.id, None)?;
            output::write(&query.output, formatter.as_ref(), &query_output)?;

            // render and process tests
            let prep_stmt = match metadata.placeholder {
                Placeholder::PosArgs => Some(query_output.as_str()),
                Placeholder::Variables => None,
            };
            for tt in metadata.test_templates.find_by_query(&query.id) {
                let test_output = engine.render_test(&tt.path, prep_stmt)?;
                output::write(&tt.output, formatter.as_ref(), &test_output)?;
            }
        }
        Ok(0)
    } else {
        println!("Invalid manifest file: '{}'", path.display());
        for mistake in mistakes {
            println!("{}", mistake.err_msg())
        }
        Ok(1)
    }
}

pub fn init(dir: &Path) -> Result<i32, Error> {
    match scaffolding::init_project(dir) {
        Ok(()) => {
            println!("New tapestry project initialized at: {}", dir.display());
            Ok(0)
        }
        Err(Error::Scaffolding(emsg)) => {
            eprintln!("Error initializing new tapestry project");
            eprintln!("Reason: {emsg}");
            Ok(1)
        }
        Err(e) => Err(e),
    }
}

pub fn summary() -> Result<i32, Error> {
    let path = Path::new("tapestry.toml");
    let metadata = Metadata::try_from(path)?;
    let mistakes = metadata.validate();
    if mistakes.is_empty() {
        let header = vec!["Id", "Query", "Template", "Tests"];
        let mut rows: Vec<Vec<String>> = Vec::with_capacity(metadata.queries.len());
        for query in metadata.queries.iter() {
            let id = query.id.clone();
            let path = query.output.display().to_string();
            let template_path = query.template.display().to_string();
            let tests = metadata
                .test_templates
                .find_by_query(&id)
                .iter()
                .map(|t| t.output.display().to_string())
                .collect::<Vec<String>>()
                .join("\n");

            rows.push(vec![id, path, template_path, tests]);
        }
        let mut table = Table::new();
        table.set_header(header).add_rows(rows);
        println!("{table}");
        Ok(0)
    } else {
        println!("Invalid manifest file: '{}'", path.display());
        for mistake in mistakes {
            println!("{}", mistake.err_msg())
        }
        Ok(1)
    }
}

pub fn status(assert_no_changes: bool) -> Result<i32, Error> {
    let path = Path::new("tapestry.toml");
    let metadata = Metadata::try_from(path)?;
    let mistakes = metadata.validate();
    if mistakes.is_empty() {
        let engine = Engine::from(&metadata);
        let formatter = &metadata.formatter;
        let mut stats: HashMap<&Path, output::Status> = HashMap::new();
        for query in metadata.queries.iter() {
            let q_output = engine.render_query(&query.id, None)?;
            let q_stat = output::status(&query.output, formatter.as_ref(), &q_output)?;
            println!("Query: {}: {}", &q_stat.label(), query.output.display());
            stats.insert(&query.output, q_stat);

            // render and process tests
            let prep_stmt = match metadata.placeholder {
                Placeholder::PosArgs => Some(q_output.as_str()),
                Placeholder::Variables => None,
            };
            for tt in metadata.test_templates.find_by_query(&query.id) {
                let t_output = engine.render_test(&tt.path, prep_stmt)?;
                let t_stat = output::status(&tt.output, formatter.as_ref(), &t_output)?;
                println!("  Test: {}: {}", &t_stat.label(), &tt.output.display());
                stats.insert(&tt.output, t_stat);
            }
        }
        let exit_code = if assert_no_changes {
            let no_changes = stats.values()
                .all(|status| *status == output::Status::Unchanged);
            if no_changes { 0 } else { 1 }
        } else {
            0
        };
        Ok(exit_code)
    } else {
        println!("Invalid manifest file: '{}'", path.display());
        for mistake in mistakes {
            println!("{}", mistake.err_msg())
        }
        Ok(1)
    }
}
