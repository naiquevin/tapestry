use crate::error::Error;
use crate::metadata::Metadata;
use crate::sql_format::{Formatter, PgFormatter};
use minijinja::Environment;
use serde::Serialize;
use std::convert::From;
use std::fs;
use std::path::Path;

fn create_project_dir(path: &Path) -> Result<(), Error> {
    match path.try_exists() {
        Ok(true) => Err(Error::Scaffolding("Dir already exists".to_owned())),
        Ok(false) => {
            fs::create_dir(path).map_err(Error::Io)?;
            Ok(())
        },
        Err(e) => Err(Error::Io(e))
    }
}

#[derive(Serialize)]
struct PgFormatterContext<'a> {
    exec_path: &'a Path,
    conf_path: Option<&'a Path>,
}

impl<'a> From<&'a PgFormatter> for PgFormatterContext<'a> {
    fn from(source: &'a PgFormatter) -> Self {
        Self {
            exec_path: source.exec_path.as_path(),
            conf_path: source.conf_path.as_ref().map(|p| p.as_path()),
        }
    }
}

#[derive(Serialize)]
struct DefaultManifestContext<'a> {
    placeholder: &'a str,
    query_templates_dir: &'a Path,
    test_templates_dir: &'a Path,
    queries_output_dir: &'a Path,
    tests_output_dir: &'a Path,
    pg_format: Option<PgFormatterContext<'a>>,
}

impl<'a> From<&'a Metadata> for DefaultManifestContext<'a> {
    fn from(m: &'a Metadata) -> Self {
        let pg_format = m.formatter.as_ref().and_then(|formatter| {
            match formatter {
                Formatter::PgFormatter(pgf) => Some(PgFormatterContext::from(pgf))
            }
        });
        Self {
            placeholder: m.placeholder.label(),
            query_templates_dir: m.query_templates_dir.as_path(),
            test_templates_dir: m.test_templates_dir.as_path(),
            queries_output_dir: m.queries_output_dir.as_path(),
            tests_output_dir: m.tests_output_dir.as_path(),
            pg_format,
        }
    }
}

fn write_manifest(path: &Path, metadata: &Metadata) -> Result<(), Error> {
    let mut env = Environment::new();
    env.add_template("manifest", include_str!("../defaults/manifest.toml.jinja"))
        .map_err(Error::MiniJinja)?;
    let template = env.get_template("manifest").map_err(Error::MiniJinja)?;
    let ctx = DefaultManifestContext::from(metadata);
    let content = template.render(ctx).unwrap();
    fs::write(path, content).map_err(Error::Io)?;
    Ok(())
}

pub fn init_project(dir: &Path) -> Result<(), Error> {
    // Create the project root dir
    create_project_dir(dir)?;

    // Default metadata
    let mut metadata = Metadata::default();

    // Check if any formating tool is installed on the system
    metadata.formatter = Formatter::discover();

    // Create the manifest file
    let manifest_path = dir.join("tapestry.toml");
    write_manifest(manifest_path.as_path(), &metadata)?;

    // Create subdirs
    fs::create_dir_all(dir.join(&metadata.query_templates_dir)).map_err(Error::Io)?;
    fs::create_dir_all(dir.join(&metadata.test_templates_dir)).map_err(Error::Io)?;

    Ok(())
}

