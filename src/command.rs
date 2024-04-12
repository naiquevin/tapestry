use crate::error::Error;
use crate::metadata::Metadata;
use crate::output;
use crate::placeholder::Placeholder;
use crate::render::Engine;
use std::path::Path;

pub fn validate() -> Result<(), Error> {
    let path = Path::new("tapestry.toml");
    let metadata = Metadata::try_from(path)?;
    let mistakes = metadata.validate();
    if mistakes.is_empty() {
        println!("All Ok: Manifest file '{}' is valid", path.display());
        Ok(())
    } else {
        println!("Invalid manifest file: '{}'", path.display());
        for mistake in mistakes {
            println!("{}", mistake.err_msg())
        }
        Err(Error::InvalidManifest)
    }
}

pub fn render() -> Result<(), Error> {
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

            // render and process test
            let prep_stmt = match metadata.placeholder {
                Placeholder::PosArgs => Some(query_output.as_str()),
                Placeholder::Variables => None,
            };
            for tt in metadata.test_templates.find_by_query(&query.id) {
                let test_output = engine.render_test(&tt.path, prep_stmt)?;
                output::write(&tt.output, formatter.as_ref(), &test_output)?;
            }
        }
        Ok(())
    } else {
        println!("Invalid manifest file: '{}'", path.display());
        for mistake in mistakes {
            println!("{}", mistake.err_msg())
        }
        Err(Error::InvalidManifest)
    }
}
