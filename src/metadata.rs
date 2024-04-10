use crate::error::{parse_error, Error};
use crate::placeholder::Placeholder;
use crate::query::Queries;
use crate::query_template::QueryTemplates;
use crate::sql_format::Formatter;
use crate::test_template::TestTemplates;
use crate::toml::decode_pathbuf;
use crate::validation::{validate_path, ManifestMistake};
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use toml::Table;

#[allow(unused)]
#[derive(Debug)]
pub struct Metadata {
    pub placeholder: Placeholder,
    pub query_templates_dir: PathBuf,
    pub test_templates_dir: PathBuf,
    pub formatter: Option<Formatter>,
    queries_output_dir: PathBuf,
    tests_output_dir: PathBuf,
    pub query_templates: QueryTemplates,
    pub queries: Queries,
    pub test_templates: TestTemplates,
}

impl TryFrom<&Path> for Metadata {
    type Error = Error;

    fn try_from(p: &Path) -> Result<Self, Self::Error> {
        let contents = std::fs::read_to_string(p).map_err(Error::Io)?;
        let table: Table = contents.parse().map_err(Error::Toml)?;
        let placeholder = table
            .get("placeholder")
            .ok_or(parse_error!("Key 'placeholder' is missing"))
            .map(Placeholder::try_from)??;
        let query_templates_dir = table
            .get("query_templates_dir")
            .ok_or(parse_error!("Key 'query_templates_dir' is missing"))
            .map(|v| decode_pathbuf(v, None, "query_templates_dir"))??;
        let test_templates_dir = table
            .get("test_templates_dir")
            .ok_or(parse_error!("Key 'test_templates_dir' is missing"))
            .map(|v| decode_pathbuf(v, None, "test_templates_dir"))??;
        let queries_output_dir = table
            .get("queries_output_dir")
            .ok_or(parse_error!("Key 'queries_output_dir' is missing"))
            .map(|v| decode_pathbuf(v, None, "query_output_dir"))??;
        let tests_output_dir = table
            .get("tests_output_dir")
            .ok_or(parse_error!("Key 'tests_output_dir' is missing"))
            .map(|v| decode_pathbuf(v, None, "tests_output_dir"))??;

        let formatter = match table.get("formatter").map(Formatter::decode) {
            Some(res) => res?,
            None => None,
        };

        let query_templates = match table.get("query_templates") {
            Some(v) => QueryTemplates::decode(&query_templates_dir, v)?,
            // @TODO: Log a warning here as there is nothing to be
            // done if no templates are defined.
            None => QueryTemplates::new(),
        };

        let queries = match table.get("queries") {
            Some(v) => Queries::decode(&query_templates_dir, &queries_output_dir, v)?,
            // @TODO: Log a warning here as there is nothing to be
            // done if no queries are defined.
            None => Queries::new(),
        };

        let test_templates = match table.get("test_templates") {
            Some(v) => TestTemplates::decode(&test_templates_dir, &tests_output_dir, v)?,
            // @TODO: Log a warning here as there is nothing to be
            // done if no queries are defined.
            None => TestTemplates::new(),
        };

        let m = Self {
            placeholder,
            query_templates_dir,
            test_templates_dir,
            queries_output_dir,
            tests_output_dir,
            formatter,
            query_templates,
            queries,
            test_templates,
        };

        Ok(m)
    }
}

impl Metadata {
    pub fn validate(&self) -> Vec<ManifestMistake> {
        let mut mistakes = vec![];
        match validate_path(&self.query_templates_dir, "query_templates_dir") {
            Ok(()) => {}
            Err(m) => mistakes.push(m),
        }
        match validate_path(&self.test_templates_dir, "test_templates_dir") {
            Ok(()) => {}
            Err(m) => mistakes.push(m),
        }
        mistakes.append(&mut self.query_templates.validate());
        mistakes.append(&mut self.queries.validate(&self.query_templates));
        mistakes.append(&mut self.test_templates.validate(&self.queries));
        mistakes
    }
}
