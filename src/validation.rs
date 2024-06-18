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
        template: &'a str,
    },
    QueryRefNotFound {
        query_id: &'a str,
        test_template: &'a str,
    },
    InvalidConds {
        query_id: &'a str,
        conds: Vec<&'a str>,
    },
    Duplicates {
        key: &'a str,
        value: &'a str,
    },
    InvalidQueryOutput {
        query_id: &'a str,
        output_path: &'a Path,
    },
    DisparateQueryOutputs,
}

impl<'a> ManifestMistake<'a> {
    pub fn err_msg(&self) -> String {
        match self {
            Self::PathDoesnotExist { path, key } => {
                let path_str = path.to_str().unwrap();
                format!("Path '{path_str}' does not exist; key: '{key}'")
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
