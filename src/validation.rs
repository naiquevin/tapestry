// FIX: Use Cow<str> in error payloads to support non-UTF8 paths without panicking.
use std::borrow::Cow;
use std::path::Path;

#[derive(Debug)]
pub enum ManifestMistake<'a> {
    PathDoesnotExist {
        path: &'a Path,
        key: &'a str,
    },
    InvalidOutputDir {
        path: &'a Path,
        key: &'a str,
    },
    NonUniqueDirs,
    QueryTemplateRefNotFound {
        query_id: &'a str,
        // FIX: Template path may be non-UTF8; store as Cow<str> to avoid unwrap.
        template: Cow<'a, str>,
    },
    QueryRefNotFound {
        query_id: &'a str,
        // FIX: Test template path may be non-UTF8; store as Cow<str> to avoid unwrap.
        test_template: Cow<'a, str>,
    },
    InvalidConds {
        query_id: &'a str,
        conds: Vec<&'a str>,
    },
    Duplicates {
        key: &'a str,
        // FIX: Can be owned (e.g. from Path::display()) to avoid UTF-8 assumptions.
        value: Cow<'a, str>,
    },
    InvalidQueryOutput {
        query_id: &'a str,
        output_path: &'a Path,
    },
    DisparateQueryOutputs,
    NameTaggingRequired(String),
}

impl<'a> ManifestMistake<'a> {
    pub fn err_msg(&self) -> String {
        match self {
            Self::PathDoesnotExist { path, key } => {
                format!("Path '{}' does not exist; key: '{key}'", path.display())
            }
            Self::InvalidOutputDir { path, key } => {
                format!(
                    "Invalid output dir path {key} = {} (must not be root or empty)",
                    path.display()
                )
            }
            Self::NonUniqueDirs => {
                "Values for all '*_dir' keys in the manifest file must be unique".to_string()
            }
            Self::QueryTemplateRefNotFound { query_id, template } => {
                format!("Query '{query_id}' refers to unknown template: '{template}'")
            }
            Self::QueryRefNotFound {
                query_id,
                test_template,
            } => {
                format!("Test template '{test_template}' refers to unknown query '{query_id}'")
            }
            Self::InvalidConds { query_id, conds } => {
                format!("Invalid 'conds': {conds:?} defined for query: '{query_id}'")
            }
            Self::Duplicates { key, value } => {
                format!("Duplicates found; key: '{key}', value: '{value}'")
            }
            Self::InvalidQueryOutput { query_id, output_path } => {
                format!(
                    "Query output for '{query_id}' is {}; Expected to be same as 'query_output_file' when layout = one-file-all-queries",
                    output_path.display(),
                )
            },
            Self::DisparateQueryOutputs => {
                String::from("Disparate query outputs found. All expected to be same as 'query_output_file' when layout = one-file-all-queries")
            },
            Self::NameTaggingRequired(reason) => {
                format!("Name tagging is required for reason: {reason}")
            }
        }
    }
}

pub fn validate_path<'a>(path: &'a Path, key: &'a str) -> Result<(), ManifestMistake<'a>> {
    match path.try_exists() {
        Ok(true) => Ok(()),
        Ok(false) => Err(ManifestMistake::PathDoesnotExist { key, path }),
        Err(_) => Err(ManifestMistake::PathDoesnotExist { key, path }),
    }
}
