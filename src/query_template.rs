use crate::error::Error;
use crate::toml::{decode_pathbuf, decode_vecstr};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use toml::Value;

#[allow(unused)]
#[derive(Debug)]
struct QueryTemplate {
    path: PathBuf,
    all_conds: Vec<String>,
}

impl QueryTemplate {

    fn decode<P: AsRef<Path>>(base_dir: P, value: &Value) -> Result<Self, Error> {
        match value.as_table() {
            Some(t) => {
                let path = t.get("path")
                    .ok_or(Error::Parsing("Query template path missing".to_owned()))
                    .map(|v| decode_pathbuf(v, Some(base_dir.as_ref())))??;
                let all_conds = t.get("all_conds")
                    .ok_or(Error::Parsing("Invalid query tempalte".to_owned()))
                    .map(decode_vecstr)??;

                Ok(Self { path, all_conds })
            }
            None => {
                Err(Error::Parsing("Invalid 'query_template' entry".to_owned()))
            }
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
}

#[allow(unused)]
#[derive(Debug)]
pub struct QueryTemplates {
    inner: Vec<Rc<QueryTemplate>>,
    cache: HashMap<String, Rc<QueryTemplate>>
}

impl QueryTemplates {

    pub fn new() -> Self {
        let inner: Vec<Rc<QueryTemplate>> = vec![];
        let cache: HashMap<String, Rc<QueryTemplate>> = HashMap::new();
        Self { inner, cache }
    }

    pub fn decode<P: AsRef<Path>>(
        base_dir: P,
        value: &Value
    ) -> Result<Self, Error> {
        let items = match value.as_array() {
            Some(xs) => {
                let mut res = Vec::with_capacity(xs.len());
                for x in xs {
                    let qt = Rc::new(QueryTemplate::decode(&base_dir, x)?);
                    res.push(qt)
                }
                res
            }
            None => return Err(Error::Parsing("Invalid query templates".to_owned()))
        };
        let cache: HashMap<String, Rc<QueryTemplate>> = HashMap::new();
        Ok(Self { inner: items, cache })
    }

    #[allow(dead_code)]
    fn get(&mut self, path: &Path) -> Option<&Rc<QueryTemplate>> {
        // @UNWRAP: Path is expected to be valid UTF-8
        let key = path.to_str().unwrap();
        let entry = self.cache.entry(key.to_owned());
        match entry {
            Entry::Occupied(o) => Some(o.into_mut()),
            Entry::Vacant(v) => {
                let res = self.inner
                    .iter()
                    .find(|entry| {
                        entry.id().as_str() == key
                    });
                if let Some(qt) = res {
                    v.insert(qt.clone());
                }
                res
            }
        }
    }
}
