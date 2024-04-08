use crate::error::{parse_error, Error};
use crate::toml::{decode_pathbuf, decode_vecstr};
use crate::validation::{validate_path, ManifestMistake};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use toml::Value;

#[allow(unused)]
#[derive(Debug)]
pub struct QueryTemplate {
    path: PathBuf,
    pub all_conds: Vec<String>,
}

impl QueryTemplate {
    fn decode<P: AsRef<Path>>(base_dir: P, value: &Value) -> Result<Self, Error> {
        match value.as_table() {
            Some(t) => {
                let path = t
                    .get("path")
                    .ok_or(parse_error!("Query template path missing"))
                    .map(|v| {
                        decode_pathbuf(v, Some(base_dir.as_ref()), "query_templates[].path")
                    })??;
                let all_conds = t
                    .get("all_conds")
                    .ok_or(parse_error!("Missing 'all_conds' in 'query_template'"))
                    .map(|v| decode_vecstr(v, "query_templates[].all_conds"))??;

                Ok(Self { path, all_conds })
            }
            None => Err(parse_error!("Invalid 'query_template' entry")),
        }
    }

    /// Returns identifier for the QueryTemplate
    ///
    /// It's simply the path returned as String. Expected to be used
    /// for caching etc.
    fn id(&self) -> String {
        // @UNWRAP: Path is expected to be valid UTF-8
        self.path.to_str().unwrap().to_owned()
    }

    fn validate(&self) -> Option<ManifestMistake> {
        match validate_path(&self.path, "query_templates[].path") {
            Ok(()) => None,
            Err(m) => Some(m),
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct QueryTemplates {
    inner: Vec<Rc<QueryTemplate>>,
    cache: HashMap<String, Rc<QueryTemplate>>,
}

impl QueryTemplates {
    pub fn new() -> Self {
        let inner: Vec<Rc<QueryTemplate>> = vec![];
        let cache: HashMap<String, Rc<QueryTemplate>> = HashMap::new();
        Self { inner, cache }
    }

    pub fn decode<P: AsRef<Path>>(base_dir: P, value: &Value) -> Result<Self, Error> {
        // @NOTE: The cache is warmed at the time of initialization to
        // avoid complexity. A lazy and memory efficient approach
        // would be populating the cache at the time of lookup but in
        // that case we'd need to manage multiple mutable references.
        let mut cache: HashMap<String, Rc<QueryTemplate>> = HashMap::new();
        let items = match value.as_array() {
            Some(xs) => {
                let mut res = Vec::with_capacity(xs.len());
                for x in xs {
                    let qt = Rc::new(QueryTemplate::decode(&base_dir, x)?);
                    let cache_key = qt.id();
                    let cache_val = qt.clone();
                    res.push(qt);
                    cache.insert(cache_key, cache_val);
                }
                res
            }
            None => return Err(parse_error!("Invalid query templates")),
        };

        Ok(Self {
            inner: items,
            cache,
        })
    }

    pub fn validate(&self) -> Vec<ManifestMistake> {
        let mut mistakes = vec![];
        let count = self.inner.len();
        let mut all_paths: HashMap<&Path, usize> = HashMap::with_capacity(count);
        for qt in &self.inner {
            if let Some(m) = qt.validate() {
                mistakes.push(m);
            }
            all_paths
                .entry(&qt.path)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
        for (key, val) in all_paths.iter() {
            if val > &1 {
                let m = ManifestMistake::Duplicates {
                    key: "query_templates[].path",
                    value: key.to_str().unwrap(),
                };
                mistakes.push(m)
            }
        }
        mistakes
    }

    pub fn get(&self, path: &Path) -> Option<&Rc<QueryTemplate>> {
        let key = path.to_str().unwrap().to_owned();
        self.cache.get(&key)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_validation_duplicates() {
        let mut qts = QueryTemplates::new();
        let qt_1 = QueryTemplate {
            path: PathBuf::from("examples/chinook/templates/queries/artists_long_songs.sql.j2"),
            all_conds: vec!["genre".to_owned(), "limit".to_owned()],
        };
        let qt_2 = QueryTemplate {
            path: PathBuf::from("examples/chinook/templates/queries/songs.sql.j2"),
            all_conds: vec![
                "artist".to_owned(),
                "file_format".to_owned(),
                "album_name".to_owned(),
            ],
        };
        let qt_3 = QueryTemplate {
            path: PathBuf::from("examples/chinook/templates/queries/artists_long_songs.sql.j2"),
            all_conds: vec!["limit".to_owned()],
        };
        qts.inner.push(Rc::new(qt_1));
        qts.inner.push(Rc::new(qt_2));
        qts.inner.push(Rc::new(qt_3));

        let mistakes = qts.validate();
        assert_eq!(1, mistakes.len());
        match mistakes[0] {
            ManifestMistake::Duplicates { key, value } => {
                assert_eq!("query_templates[].path", key);
                assert_eq!(
                    "examples/chinook/templates/queries/artists_long_songs.sql.j2",
                    value
                );
            }
            _ => assert!(false),
        }
    }
}
