use crate::error::{parse_error, Error};
use crate::placeholder::Placeholder;
use crate::query::Queries;
use crate::query_template::QueryTemplates;
use crate::sql_format::Formatter;
use crate::test_template::TestTemplates;
use crate::toml::decode_pathbuf;
use crate::util::ls_files;
use crate::validation::{validate_path, ManifestMistake};
use log::warn;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use toml::Table;

#[derive(Debug)]
pub struct Metadata {
    pub placeholder: Placeholder,
    pub query_templates_dir: PathBuf,
    pub test_templates_dir: PathBuf,
    pub formatter: Option<Formatter>,
    pub queries_output_dir: PathBuf,
    pub tests_output_dir: PathBuf,
    pub query_templates: QueryTemplates,
    pub queries: Queries,
    pub test_templates: TestTemplates,
}

/// `try_from` method for initializing `Metadata` from path to the
/// manifest file.
impl TryFrom<&Path> for Metadata {
    type Error = Error;

    fn try_from(p: &Path) -> Result<Self, Self::Error> {
        let contents = std::fs::read_to_string(p).map_err(|_| {
            // @TODO: Log the underlying `std::io::Error` at debug
            // error here
            Error::ManifestNotFound
        })?;
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
    pub fn default() -> Self {
        Self {
            placeholder: Placeholder::PosArgs,
            query_templates_dir: PathBuf::from("templates/queries"),
            test_templates_dir: PathBuf::from("templates/tests"),
            formatter: None,
            queries_output_dir: PathBuf::from("output/queries"),
            tests_output_dir: PathBuf::from("output/tests"),
            query_templates: QueryTemplates::new(),
            queries: Queries::new(),
            test_templates: TestTemplates::new(),
        }
    }

    /// Logs warnings when certain conditions where we don't want to
    /// invalidate the command, but simply let the user know that
    /// something may not be as per expectation
    fn warnings(&self) -> Result<(), Error> {
        // Warn regarding unused query templates (i.e. when a query
        // template is defined in the manifest but there's no query
        // defined that uses it)
        let qt_defined: HashSet<&Path> = self
            .query_templates
            .iter()
            .map(|qt| qt.path.as_ref())
            .collect();
        let qt_used: HashSet<&Path> = self.queries.iter().map(|q| q.template.as_ref()).collect();
        let qt_unused = qt_defined.difference(&qt_used);
        for qt in qt_unused {
            warn!("Unused query template found in manifest: {}", qt.display());
        }

        // Warn regarding undefined query template files i.e. the
        // query template files that exist in the
        // `query_templates_dir` but not defined in the manifest. This
        // will happen when the user creates query template file but
        // forgets to specify it in the manifest
        let qt_files = ls_files(&self.query_templates_dir).map_err(Error::Io)?;
        let qt_actual: HashSet<&Path> = qt_files.iter().map(|p| p.as_ref()).collect();
        let qt_undefined = qt_actual.difference(&qt_defined);
        for qt in qt_undefined {
            warn!(
                "Did you miss defining query template in manifest? {}",
                qt.display()
            );
        }

        // Warn regarding undefined test template files i.e. the test
        // template files that exist in the `test_templates_dir` but
        // not defined in the manifest. This will happen when the user
        // creates a test template file but forgets to specify it in
        // the manifest.
        let tt_defined: HashSet<&Path> = self
            .test_templates
            .iter()
            .map(|tt| tt.path.as_ref())
            .collect();
        let tt_files = ls_files(&self.test_templates_dir).map_err(Error::Io)?;
        let tt_actual: HashSet<&Path> = tt_files.iter().map(|p| p.as_ref()).collect();
        let tt_undefined = tt_actual.difference(&tt_defined);
        for tt in tt_undefined {
            warn!(
                "Did you miss defining test template in manifest? {}",
                tt.display()
            );
        }
        Ok(())
    }

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

        if self.queries_output_dir.parent().is_none() {
            mistakes.push(ManifestMistake::InvalidOutputDir {
                path: &self.queries_output_dir,
                key: "queries_output_dir",
            })
        }

        if self.tests_output_dir.parent().is_none() {
            mistakes.push(ManifestMistake::InvalidOutputDir {
                path: &self.tests_output_dir,
                key: "tests_output_dir",
            })
        }

        let all_dirs = HashSet::from([
            &self.query_templates_dir,
            &self.test_templates_dir,
            &self.queries_output_dir,
            &self.tests_output_dir,
        ]);

        if all_dirs.len() < 4 {
            mistakes.push(ManifestMistake::NonUniqueDirs);
        }

        mistakes.append(&mut self.query_templates.validate());
        mistakes.append(&mut self.queries.validate(&self.query_templates));
        mistakes.append(&mut self.test_templates.validate(&self.queries));

        // Log warnings if any
        let _ = self.warnings();

        mistakes
    }
}
