use std::path::Path;

#[derive(Debug)]
pub enum ManifestMistake<'a> {
    PathDoesnotExist {
        path: &'a Path,
        key: &'a str,
    },
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
}

pub fn validate_path<'a>(path: &'a Path, key: &'a str) -> Result<(), ManifestMistake<'a>> {
    match path.try_exists() {
        Ok(true) => Ok(()),
        Ok(false) => Err(ManifestMistake::PathDoesnotExist { key, path }),
        Err(_) => Err(ManifestMistake::PathDoesnotExist { key, path }),
    }
}
