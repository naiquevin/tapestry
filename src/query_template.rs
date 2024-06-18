use crate::error::{parse_error, Error};
use crate::toml::{decode_pathbuf, decode_strset};
use crate::validation::{validate_path, ManifestMistake};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use toml::Value;

#[derive(Debug)]
pub struct QueryTemplate {
    pub path: PathBuf,
    pub all_conds: HashSet<String>,
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
                let all_conds = match t.get("all_conds") {
                    Some(v) => decode_strset(v, "query_templates[].all_conds")?,
                    None => HashSet::new(),
                };
                Ok(Self { path, all_conds })
            }
            None => Err(parse_error!("Invalid 'query_template' entry")),
        }
    }

    /// Returns identifier for the QueryTemplate
    ///
    /// It's simply the path returned as String. Expected to be used
    /// for indexing etc.
    fn id(&self) -> &str {
        // @UNWRAP: Path is expected to be valid UTF-8
        self.path.to_str().unwrap()
    }

    /// Returns file name of the template which can be used with
    /// `minijinja::Environment` that's initialized using
    /// `minijinja::path_loader`
    ///
    /// # Panics
    ///
    /// 1. This fn assumes that the template path is valid unicode and
    /// will panic if that's not the case.
    ///
    /// 2. If the path ends in `..`
    ///
    pub fn file_name(&self) -> &str {
        self.path
            .file_name()
            .map(|ostr| ostr.to_str().unwrap())
            .unwrap()
    }

    fn validate(&self) -> Option<ManifestMistake> {
        match validate_path(&self.path, "query_templates[].path") {
            Ok(()) => None,
            Err(m) => Some(m),
        }
    }
}

#[derive(Debug)]
pub struct QueryTemplates {
    inner: Vec<Rc<QueryTemplate>>,
    index: HashMap<String, Rc<QueryTemplate>>,
}

impl QueryTemplates {
    pub fn new() -> Self {
        let inner: Vec<Rc<QueryTemplate>> = vec![];
        let index: HashMap<String, Rc<QueryTemplate>> = HashMap::new();
        Self { inner, index }
    }

    pub fn decode<P: AsRef<Path>>(base_dir: P, value: &Value) -> Result<Self, Error> {
        // @NOTE: The index is populated at the time of initialization
        // to avoid complexity. A lazy and memory efficient approach
        // would be populating the index at the time of lookup (like a
        // read-through cache) but in that case we'd need to manage
        // multiple mutable references.
        let mut index: HashMap<String, Rc<QueryTemplate>> = HashMap::new();
        let items = match value.as_array() {
            Some(xs) => {
                let mut res = Vec::with_capacity(xs.len());
                for x in xs {
                    let qt = Rc::new(QueryTemplate::decode(&base_dir, x)?);
                    let idx_key = qt.id().to_owned();
                    let idx_val = qt.clone();
                    res.push(qt);
                    index.insert(idx_key, idx_val);
                }
                res
            }
            None => return Err(parse_error!("Invalid query templates")),
        };

        Ok(Self {
            inner: items,
            index,
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
        self.index.get(&key)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Rc<QueryTemplate>> {
        self.inner.iter()
    }
}

#[cfg(test)]
pub mod test_util {

    use super::*;

    pub fn strset(xs: Vec<&str>) -> HashSet<String> {
        xs.iter().map(|s| String::from(*s)).collect()
    }

    // Setup query_templates from &'static str input to be used in
    // tests
    //
    // Argument `data` is a vector of tuple with following type of
    // values (in order):
    //
    //   path: &str, all_conds: Vec<&str>
    pub fn setup_query_templates(data: Vec<(&str, Vec<&str>)>) -> QueryTemplates {
        let mut qts = QueryTemplates::new();
        for (p, ac) in data {
            let qt = Rc::new(QueryTemplate {
                path: PathBuf::from(p),
                all_conds: strset(ac),
            });
            let idx_key = qt.id().to_owned();
            let idx_val = qt.clone();
            qts.inner.push(qt);
            qts.index.insert(idx_key, idx_val);
        }
        qts
    }
}

#[cfg(test)]
mod tests {

    use super::test_util::*;
    use super::*;
    use toml;

    #[test]
    fn test_decode_query_template() {
        // When TOML is valid
        let table = r#"
path = 'my_query_template.sql.j2'
all_conds = [ 'foo', 'bar' ]
"#
        .parse::<toml::Table>()
        .unwrap();
        let value = toml::Value::Table(table);
        match QueryTemplate::decode("base", &value) {
            Ok(qt) => {
                assert_eq!(PathBuf::from("base/my_query_template.sql.j2"), qt.path);
                assert_eq!(strset(vec!["foo", "bar"]), qt.all_conds);
            }
            Err(_) => assert!(false),
        }

        // When `path` is missing
        let table = r#"
all_conds = [ 'foo', 'bar' ]
"#
        .parse::<toml::Table>()
        .unwrap();
        let value = toml::Value::Table(table);
        match QueryTemplate::decode("base", &value) {
            Ok(_) => assert!(false),
            Err(Error::Parsing(msg)) => {
                assert_eq!("Query template path missing", msg);
            }
            Err(_) => assert!(false),
        }

        // When `all_conds` is missing
        let table = r#"
path = 'my_query_template.sql.j2'
"#
        .parse::<toml::Table>()
        .unwrap();
        let value = toml::Value::Table(table);
        match QueryTemplate::decode("base", &value) {
            Ok(qt) => {
                assert_eq!(PathBuf::from("base/my_query_template.sql.j2"), qt.path);
                assert_eq!(HashSet::new(), qt.all_conds)
            }
            Err(_) => assert!(false),
        }

        // When type of TOML value is not a table
        let value = toml::Value::String(String::from("hello"));
        match QueryTemplate::decode("base", &value) {
            Ok(_) => assert!(false),
            Err(Error::Parsing(msg)) => {
                assert_eq!("Invalid 'query_template' entry", msg);
            }
            Err(_) => assert!(false),
        }

        // When `all_conds` is not of correct type
        let table = r#"
path = 'my_query_template.sql.j2'
all_conds = [ 1, 2 ]
"#
        .parse::<toml::Table>()
        .unwrap();
        let value = toml::Value::Table(table);
        match QueryTemplate::decode("base", &value) {
            Ok(_) => assert!(false),
            Err(Error::Parsing(msg)) => {
                assert_eq!(
                    "Value of 'query_templates[].all_conds' is expected to be array of strings",
                    msg
                );
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_validation_duplicates() {
        let qts = setup_query_templates(vec![
            (
                "examples/chinook/templates/queries/artists_long_songs.sql.j2",
                vec!["genre", "limit"],
            ),
            (
                "examples/chinook/templates/queries/songs_formats.sql.j2",
                vec!["genre", "limit"],
            ),
            (
                "examples/chinook/templates/queries/artists_long_songs.sql.j2",
                vec!["limit"],
            ),
        ]);

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
