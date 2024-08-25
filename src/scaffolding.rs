use crate::error::Error;
use crate::formatters::Formatter;
use crate::metadata::Metadata;
use crate::tagging::NameTagger;
use crate::toml::SerializableTomlTable;
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
        }
        Err(e) => Err(Error::Io(e)),
    }
}

#[derive(Serialize)]
struct NameTaggerContext {
    style: String,
}

impl From<&NameTagger> for NameTaggerContext {
    fn from(tagger: &NameTagger) -> Self {
        Self {
            style: tagger.style.to_string(),
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
    formatter: Option<SerializableTomlTable>,
    name_tagger: Option<NameTaggerContext>,
}

impl<'a> From<&'a Metadata> for DefaultManifestContext<'a> {
    fn from(m: &'a Metadata) -> Self {
        let formatter = m.formatter.as_ref().and_then(|f| f.config_toml_table());
        let name_tagger = m.name_tagger.as_ref().map(NameTaggerContext::from);
        Self {
            placeholder: m.placeholder.label(),
            query_templates_dir: m.query_templates_dir.as_path(),
            test_templates_dir: m.test_templates_dir.as_path(),
            queries_output_dir: m.queries_output_dir.as_path(),
            tests_output_dir: m.tests_output_dir.as_path(),
            formatter,
            name_tagger,
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

fn write_formatter_configs(dir: &Path, formatter: Option<&Formatter>) -> Result<(), Error> {
    if let Some(f) = formatter {
        match f {
            Formatter::PgFormatter(pgf) => {
                if let Some(p) = &pgf.conf_path {
                    let conf_path = dir.join(p);
                    fs::create_dir(conf_path.parent().unwrap()).map_err(Error::Io)?;
                    fs::write(conf_path, include_str!("../defaults/pg_format.config"))
                        .map_err(Error::Io)?;
                }
            }
            Formatter::SqlFormatRs(_) => {}
        }
    }
    Ok(())
}

pub fn init_project(dir: &Path) -> Result<(), Error> {
    // Create the project root dir
    create_project_dir(dir)?;

    // Default metadata
    let mut metadata = Metadata::default();

    // Check if any formating tool is installed on the system
    metadata.formatter = Formatter::discover();

    // @TODO: Here we will allow the user to choose the formatter
    // smartly as follows:
    //
    //  1. We will find out which all formatters supported by tapestry
    //     are installed on the user's system and collect them into a
    //     list
    //
    //  2. By default, the 'sqlformat-rs' option will be added to this
    //     list. This is to handle the case where the user has no
    //     external formatters installed on the system
    //
    //  3. A `None` option will also be added to this list, in case
    //     the user prefers to not format the sql at all (why?)
    //
    //  4. Then we will prompt the user to select one among them (the
    //     inquire crate can be used for this).
    //
    //  5. Config for the selected formatter will be added to the
    //     manifest file.
    //

    // Create the manifest file
    let manifest_path = dir.join("tapestry.toml");
    write_manifest(manifest_path.as_path(), &metadata)?;

    // Create subdirs
    fs::create_dir_all(dir.join(&metadata.query_templates_dir)).map_err(Error::Io)?;
    fs::create_dir_all(dir.join(&metadata.test_templates_dir)).map_err(Error::Io)?;

    // Create formatter config files if applicable
    write_formatter_configs(dir, metadata.formatter.as_ref())?;

    Ok(())
}
