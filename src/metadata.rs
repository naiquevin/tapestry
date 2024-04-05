use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::convert::TryFrom;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use toml::{Table, Value};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Toml(toml::de::Error),
    Parsing(String),
}

#[derive(Debug)]
enum Placeholder {
    PosArgs,
    Variables
}

impl TryFrom<&Value> for Placeholder {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_str() {
            Some(s) => {
                if s == "posargs" {
                    Ok(Self::PosArgs)
                } else if s == "variables" {
                    Ok(Self::Variables)
                } else {
                    let msg = format!("Invalid placeholder: '{s}'");
                    Err(Error::Parsing(msg))
                }
            }
            None => {
                Err(Error::Parsing("Value of key 'placeholder' must be a string".to_owned()))
            }
        }
    }
}

fn decode_string(value: &Value) -> Result<String, Error> {
    value.as_str()
        .ok_or(Error::Parsing("Value expected to be a string".to_owned()))
        .map(|s| s.to_owned())
}

fn decode_pathbuf(value: &Value, base_dir: Option<&Path>) -> Result<PathBuf, Error> {
    value.as_str()
        .ok_or(Error::Parsing("Value expected to be a string".to_owned()))
        .map(|s| base_dir.map_or_else(|| PathBuf::from(s), |p| p.join(s)))
}

fn decode_vecstr(value: &Value) -> Result<Vec<String>, Error> {
    match value.as_array() {
        Some(xs) => {
            let mut res = Vec::with_capacity(xs.len());
            for v in xs {
                match v.as_str() {
                    Some(x) => res.push(x.to_owned()),
                    None => return Err(Error::Parsing("Value expected to be a string".to_owned()))
                }
            }
            Ok(res)
        }
        None => Err(Error::Parsing("Value expected to be an array".to_owned()))
    }
}

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
struct QueryTemplates {
    inner: Vec<Rc<QueryTemplate>>,
    cache: HashMap<String, Rc<QueryTemplate>>
}

impl QueryTemplates {

    fn new() -> Self {
        let inner: Vec<Rc<QueryTemplate>> = vec![];
        let cache: HashMap<String, Rc<QueryTemplate>> = HashMap::new();
        Self { inner, cache }
    }

    fn decode<P: AsRef<Path>>(
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

#[allow(unused)]
#[derive(Debug)]
struct Query {
    id: String,
    template: PathBuf,
    conds: Vec<String>,
    output: Option<PathBuf>,
}

impl Query {

    fn decode<P: AsRef<Path>>(
        templates_base_dir: P,
        output_base_dir: P,
        value: &Value
    ) -> Result<Self, Error> {
        match value.as_table() {
            Some(t) => {
                let id = t.get("id")
                    .ok_or(Error::Parsing("Missing 'id' in 'query' entry".to_owned()))
                    .map(decode_string)??;
                let template = t.get("template")
                    .ok_or(Error::Parsing("Missing 'template' in 'query' entry".to_owned()))
                    .map(|v| decode_pathbuf(v, Some(templates_base_dir.as_ref())))??;
                let conds = t.get("conds")
                    .ok_or(Error::Parsing("Missing 'conds' in 'query' entry".to_owned()))
                    .map(decode_vecstr)??;
                let output = match t.get("option") {
                    Some(v) => Some(decode_pathbuf(v, Some(output_base_dir.as_ref()))?),
                    None => None
                };
                Ok(Self { id, template, conds, output })
            },
            None => Err(Error::Parsing("Invalid 'query' entry".to_owned()))
        }
    }
}

fn decode_queries<P: AsRef<Path>>(
    templates_base_dir: P,
    output_base_dir: P,
    value: &Value
) -> Result<Vec<Query>, Error> {
    match value.as_array() {
        Some(xs) => {
            let mut res = Vec::with_capacity(xs.len());
            for x in xs {
                let q = Query::decode(&templates_base_dir, &output_base_dir, x)?;
                res.push(q);
            }
            Ok(res)
        }
        None => Err(Error::Parsing("Invalid queries".to_owned()))
    }
}

#[allow(unused)]
#[derive(Debug)]
struct TestTemplate {
    query: String,
    template: PathBuf,
    output: Option<PathBuf>,
}

impl TestTemplate {

    fn decode<P: AsRef<Path>>(
        templates_base_dir: P,
        output_base_dir: P,
        value: &Value,
    ) -> Result<Self, Error> {
        match value.as_table() {
            Some(t) => {
                let query = t.get("query")
                    .ok_or(Error::Parsing("Missing 'query' in 'test_templates' entry".to_owned()))
                    .map(decode_string)??;
                let template = t.get("template")
                    .ok_or(Error::Parsing("Missing 'template' in 'test_templates' entry".to_owned()))
                    .map(|v| decode_pathbuf(v, Some(templates_base_dir.as_ref())))??;
                let output = match t.get("output") {
                    Some(v) => Some(decode_pathbuf(v, Some(output_base_dir.as_ref()))?),
                    None => None,
                };
                Ok(Self { query, template, output })
            },
            None => Err(Error::Parsing("Invalid 'test_templates' entry".to_owned()))
        }
    }
}

fn decode_test_templates<P: AsRef<Path>>(
    templates_base_dir: P,
    output_base_dir: P,
    value: &Value
) -> Result<Vec<TestTemplate>, Error> {
    match value.as_array() {
        Some(xs) => {
            let mut res = Vec::with_capacity(xs.len());
            for x in xs {
                let tt = TestTemplate::decode(&templates_base_dir, &output_base_dir, x)?;
                res.push(tt);
            }
            Ok(res)
        },
        None => Err(Error::Parsing("Invalid test_templates".to_owned()))
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct MetaData {
    placeholder: Placeholder,
    query_templates_dir: PathBuf,
    test_templates_dir: PathBuf,
    queries_output_dir: PathBuf,
    tests_output_dir: PathBuf,
    query_templates: QueryTemplates,
    queries: Vec<Query>,
    test_templates: Vec<TestTemplate>
}

impl TryFrom<&Path> for MetaData {
    type Error = Error;

    fn try_from(p: &Path) -> Result<Self, Self::Error> {
        let contents = std::fs::read_to_string(p).map_err(Error::Io)?;
        let table: Table = contents.parse().map_err(Error::Toml)?;
        let placeholder = table.get("placeholder")
            .ok_or(Error::Parsing("Key 'placeholder' is missing".to_owned()))
            .map(Placeholder::try_from)??;
        let query_templates_dir = table.get("query_templates_dir")
            .ok_or(Error::Parsing("Key 'query_templates_dir' is missing".to_owned()))
            .map(|v| decode_pathbuf(v, None))??;
        let test_templates_dir = table.get("test_templates_dir")
            .ok_or(Error::Parsing("Key 'test_templates_dir' is missing".to_owned()))
            .map(|v| decode_pathbuf(v, None))??;
        let queries_output_dir = table.get("queries_output_dir")
            .ok_or(Error::Parsing("Key 'queries_output_dir' is missing".to_owned()))
            .map(|v| decode_pathbuf(v, None))??;
        let tests_output_dir = table.get("tests_output_dir")
            .ok_or(Error::Parsing("Key 'tests_output_dir' is missing".to_owned()))
            .map(|v| decode_pathbuf(v, None))??;

        let query_templates = match table.get("query_templates") {
            Some(v) => QueryTemplates::decode(&query_templates_dir, v)?,
            // @TODO: Log a warning here as there is nothing to be
            // done if no templates are defined.
            None => QueryTemplates::new(),
        };

        let queries = match table.get("queries") {
            Some(v) => decode_queries(&query_templates_dir, &queries_output_dir, v)?,
            // @TODO: Log a warning here as there is nothing to be
            // done if no queries are defined.
            None => vec![],
        };

        let test_templates = match table.get("test_templates") {
            Some(v) => decode_test_templates(&test_templates_dir, &tests_output_dir, v)?,
            // @TODO: Log a warning here as there is nothing to be
            // done if no queries are defined.
            None => vec![],
        };

        let m = Self {
            placeholder,
            query_templates_dir,
            test_templates_dir,
            queries_output_dir,
            tests_output_dir,
            query_templates,
            queries,
            test_templates,
        };

        Ok(m)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_placeholder_try_from() {
        let t = "placeholder = 'posargs'".parse::<Table>().unwrap();
        let p = Placeholder::try_from(&t["placeholder"]);
        assert!(p.is_ok());
        match p.unwrap() {
            Placeholder::PosArgs => assert!(true),
            _ => assert!(false),
        }

        let t = "placeholder = 'variables'".parse::<Table>().unwrap();
        let p = Placeholder::try_from(&t["placeholder"]);
        assert!(p.is_ok());
        match p.unwrap() {
            Placeholder::Variables => assert!(true),
            _ => assert!(false),
        }

        let t = "placeholder = 'question-marks'".parse::<Table>().unwrap();
        let p = Placeholder::try_from(&t["placeholder"]);
        assert!(p.is_err());
    }
}
