use crate::error::{parse_error, Error};
use crate::formatters::Formatter;
use crate::output::Layout;
use crate::placeholder::Placeholder;
use crate::query::Queries;
use crate::query_template::QueryTemplates;
use crate::tagging::{NameTagStyle, NameTagger};
use crate::test_template::TestTemplates;
use crate::toml::decode_pathbuf;
use crate::util::ls_files;
use crate::validation::{validate_path, ManifestMistake};
use log::{error, info, warn};
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
    pub query_output_layout: Layout,
    pub name_tagger: Option<NameTagger>,
    pub query_templates: QueryTemplates,
    pub queries: Queries,
    pub test_templates: TestTemplates,
}

/// `try_from` method for initializing `Metadata` from path to the
/// manifest file.
impl TryFrom<&Path> for Metadata {
    type Error = Error;

    fn try_from(p: &Path) -> Result<Self, Self::Error> {
        let contents = std::fs::read_to_string(p).map_err(|e| {
            error!("Unable to read manifest file {}: {}", p.display(), e);
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

        let query_output_layout = match table.get("query_output_layout") {
            Some(v) => Layout::decode(v, table.get("query_output_file"), &queries_output_dir)?,
            None => {
                info!("Key 'query_output_layout' not found in manifest. Using 'one-file-one-query' as the default");
                Layout::default()
            }
        };

        let name_tagger = match table.get("name_tagger") {
            Some(v) => NameTagger::decode(v)?,
            None => None,
        };

        let query_templates = match table.get("query_templates") {
            Some(v) => QueryTemplates::decode(&query_templates_dir, v)?,
            None => {
                warn!("TOML key 'query_templates' not found in manifest");
                QueryTemplates::new()
            }
        };

        let queries = match table.get("queries") {
            Some(v) => Queries::decode(
                &query_templates_dir,
                &queries_output_dir,
                &query_output_layout,
                v,
            )?,
            None => {
                warn!("TOML key 'queries' not found in manifest");
                Queries::new()
            }
        };

        let test_templates = match table.get("test_templates") {
            Some(v) => TestTemplates::decode(&test_templates_dir, &tests_output_dir, v)?,
            None => {
                warn!("TOML key 'test_templates' not found in manifest");
                TestTemplates::new()
            }
        };

        let m = Self {
            placeholder,
            query_templates_dir,
            test_templates_dir,
            queries_output_dir,
            tests_output_dir,
            formatter,
            query_output_layout,
            name_tagger,
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
            query_output_layout: Layout::default(),
            name_tagger: Some(NameTagger {
                style: NameTagStyle::KebabCase,
            }),
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
        let qt_files = ls_files(&self.query_templates_dir, false).map_err(Error::Io)?;
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
        let tt_files = ls_files(&self.test_templates_dir, false).map_err(Error::Io)?;
        let tt_actual: HashSet<&Path> = tt_files.iter().map(|p| p.as_ref()).collect();
        let tt_undefined = tt_actual.difference(&tt_defined);
        for tt in tt_undefined {
            warn!(
                "Did you miss defining test template in manifest? {}",
                tt.display()
            );
        }

        // Warn when `query_output_layout` is `one-file-all-queries`
        // and `name_tagger` is not set.
        if let Layout::OneFileAllQueries(_) = self.query_output_layout {
            if self.name_tagger.is_none() {
                warn!("Name tagging is recommended in case of 'one-file-all-queries' layout")
            }
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
        mistakes.append(
            &mut self
                .queries
                .validate(&self.query_templates, &self.query_output_layout),
        );
        mistakes.append(&mut self.test_templates.validate(&self.queries));

        // Log warnings if any
        let _ = self.warnings();

        mistakes
    }

    /// Returns the combined output file in case layout =
    /// `OneFileAllQueries`
    ///
    /// If layout is not `OneFileAllQueries`, then `None` is returned
    ///
    /// Results in an error if all queries don't have the same output
    /// path.
    ///
    /// @TODO: Add tests
    pub fn combined_output_file(&self) -> Result<Option<&Path>, Error> {
        match &self.query_output_layout {
            Layout::OneFileOneQuery => Ok(None),
            Layout::OneFileAllQueries(output_file) => {
                match output_file {
                    Some(filepath) => Ok(Some(filepath)),
                    None => {
                        let mut output_paths =
                            self.queries.output_files().collect::<HashSet<&Path>>();
                        if output_paths.len() == 1 {
                            // Unwrap is acceptable as the length is known to be 1
                            Ok(Some(output_paths.drain().next().unwrap()))
                        } else {
                            Err(Error::Layout(
                                "Common output file is required when layout = one-file-all-queries".to_string()
                            ))
                        }
                    }
                }
            }
        }
    }
}
