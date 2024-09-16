use crate::error::{parse_error, Error};
use crate::output::Layout;
use crate::query_template::QueryTemplates;
use crate::tagging::NameTag;
use crate::toml::{decode_pathbuf, decode_string, decode_strset};
use crate::validation::ManifestMistake;
use regex::Regex;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use toml::Value;

fn slugify_id(id: &str) -> Cow<'_, str> {
    let re = Regex::new(r"@|\+|&|\*").unwrap();
    re.replace_all(id, "-")
}

fn id_to_output(id: &str, base_dir: &Path) -> PathBuf {
    let filename = format!("{}.sql", &slugify_id(id));
    let filepath = PathBuf::from(filename);
    base_dir.join(filepath)
}

/// Computes the fallback value for 'output' field when one is not
/// explicitly specified.
///
/// The fallback value depends on the kind of `output_layout`.
///
///   - OneFileOneQuery: derive output file name from `id` field
///   - OneFileAllQueries: use the value associated with the enum
///     i.e. `query_output_file` set in manifest.
///
/// # Error:
/// Returns `Error::Parsing` if output_layout = `OneFileAllQueries'
/// and `query_output_file` key is not set in the manifest
fn fallback_output<P: AsRef<Path>>(
    id: &str,
    base_dir: P,
    output_layout: &Layout,
) -> Result<PathBuf, Error> {
    match output_layout {
        Layout::OneFileOneQuery => Ok(id_to_output(id, base_dir.as_ref())),
        Layout::OneFileAllQueries(output_file) => match output_file {
            Some(f) => Ok(f.to_path_buf()),
            None => Err(parse_error!(
                "Either 'queries[].output' or 'query_output_file' must be specified in case of 'one-file-all-queries' layout"
            )),
        },
    }
}

#[derive(Debug)]
pub struct Query {
    pub id: String,
    pub template: PathBuf,
    pub conds: HashSet<String>,
    pub output: PathBuf,
    pub name_tag: NameTag,
}

impl Query {
    fn decode<P: AsRef<Path>>(
        templates_base_dir: P,
        output_base_dir: P,
        output_layout: &Layout,
        value: &Value,
    ) -> Result<Self, Error> {
        match value.as_table() {
            Some(t) => {
                let id = t
                    .get("id")
                    .ok_or(parse_error!("Missing 'id' in 'query' entry"))
                    .map(|v| decode_string(v, "queries[].id"))??;
                let template = t
                    .get("template")
                    .ok_or(parse_error!("Missing 'template' in 'query' entry"))
                    .map(|v| {
                        decode_pathbuf(v, Some(templates_base_dir.as_ref()), "queries[].template")
                    })??;
                let conds = match t.get("conds") {
                    Some(v) => decode_strset(v, "queries[].conds")?,
                    None => HashSet::new(),
                };
                let output = match t.get("output") {
                    Some(v) => {
                        // @NOTE: When `output` is specified, it's
                        // added to the struct without considering the
                        // `output_layout`. For e.g. it may happen
                        // that `output_layout` is of the kind
                        // `OneFileAllQueries` and the specified
                        // `output` doesn't match the
                        // `query_output_file` in the
                        // manifest. Parsing will overlook such
                        // discrepancy but validation will catch it.
                        decode_pathbuf(v, Some(output_base_dir.as_ref()), "queries[].output")?
                    }
                    None => fallback_output(&id, output_base_dir.as_ref(), output_layout)?,
                };
                let name_tag = match t.get("name_tag") {
                    Some(v) => NameTag::Custom(decode_string(v, "queries[].name_tag")?),
                    None => NameTag::DeriveFromId(id.clone()),
                };
                Ok(Self {
                    id,
                    template,
                    conds,
                    output,
                    name_tag,
                })
            }
            None => Err(parse_error!("Invalid 'query' entry")),
        }
    }

    fn validate<'a>(&'a self, query_templates: &'a QueryTemplates) -> Vec<ManifestMistake<'a>> {
        let mut mistakes = vec![];
        match query_templates.get(&self.template) {
            Some(qt) => {
                if !self.conds.is_subset(&qt.all_conds) {
                    let diff = self
                        .conds
                        .difference(&qt.all_conds)
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>();
                    mistakes.push(ManifestMistake::InvalidConds {
                        query_id: &self.id,
                        conds: diff,
                    })
                }
            }
            None => mistakes.push(ManifestMistake::QueryTemplateRefNotFound {
                query_id: &self.id,
                template: self.template.to_str().unwrap(),
            }),
        }

        mistakes
    }

    /// Returns file name of the template, which is a `PathBuf`
    ///
    /// # Panics
    ///
    /// 1. This fn assumes that the template path is valid unicode and
    ///    will panic if that's not the case.
    ///
    /// 2. If the path ends in `..`
    ///
    pub fn template_file_name(&self) -> &str {
        self.template
            .file_name()
            .map(|ostr| ostr.to_str().unwrap())
            .unwrap()
    }
}

#[derive(Debug)]
pub struct Queries {
    inner: Vec<Rc<Query>>,
    index: HashMap<String, Rc<Query>>,
}

impl Queries {
    pub fn new() -> Self {
        let inner: Vec<Rc<Query>> = vec![];
        let index: HashMap<String, Rc<Query>> = HashMap::new();
        Self { inner, index }
    }

    pub fn decode<P: AsRef<Path>>(
        templates_base_dir: P,
        output_base_dir: P,
        output_layout: &Layout,
        value: &Value,
    ) -> Result<Self, Error> {
        // @NOTE: The index is populated at the time of initialization
        // to avoid complexity. A lazy and memory efficient approach
        // would be populating the index at the time of lookup (like a
        // read-through cache) but in that case we'd need to manage
        // multiple mutable references.
        let mut index: HashMap<String, Rc<Query>> = HashMap::new();
        let items = match value.as_array() {
            Some(xs) => {
                let mut res = Vec::with_capacity(xs.len());
                for x in xs {
                    let q = Rc::new(Query::decode(
                        &templates_base_dir,
                        &output_base_dir,
                        output_layout,
                        x,
                    )?);
                    let idx_key = q.id.clone();
                    let idx_val = q.clone();
                    res.push(q);
                    index.insert(idx_key, idx_val);
                }
                res
            }
            None => return Err(parse_error!("Invalid queries")),
        };
        Ok(Self {
            inner: items,
            index,
        })
    }

    pub fn validate<'a>(
        &'a self,
        query_templates: &'a QueryTemplates,
        output_layout: &Layout,
    ) -> Vec<ManifestMistake<'a>> {
        let mut mistakes = vec![];
        let count = self.inner.len();
        let mut all_ids: HashMap<&str, usize> = HashMap::with_capacity(count);
        let mut all_outputs: HashMap<&Path, usize> = HashMap::with_capacity(count);
        for query in &self.inner {
            mistakes.append(&mut query.validate(query_templates));
            all_ids
                .entry(&query.id)
                .and_modify(|c| *c += 1)
                .or_insert(1);
            all_outputs
                .entry(&query.output)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }

        // Validate that all 'queries[].id' values are unique
        for (key, val) in all_ids.iter() {
            if val > &1 {
                let m = ManifestMistake::Duplicates {
                    key: "queries[].id",
                    value: key,
                };
                mistakes.push(m)
            }
        }

        // Validation for 'queries[].output' depends on the layout
        match output_layout {
            Layout::OneFileOneQuery => {
                // Validate that all query entries have unique value
                // for `output`
                for (key, val) in all_outputs.iter() {
                    if val > &1 {
                        let m = ManifestMistake::Duplicates {
                            key: "queries[].output",
                            value: key.to_str().unwrap(),
                        };
                        mistakes.push(m)
                    }
                }
            }
            Layout::OneFileAllQueries(output_file) => {
                // Validate that all query entries have the same value
                // for `output` and it's the same as
                // `query_output_file` (p) if specified in the manifest
                if all_outputs.len() > 1 {
                    match output_file {
                        Some(p) => {
                            for query in &self.inner {
                                if &query.output != p {
                                    let m = ManifestMistake::InvalidQueryOutput {
                                        query_id: &query.id,
                                        output_path: &query.output,
                                    };
                                    mistakes.push(m)
                                }
                            }
                        }
                        None => mistakes.push(ManifestMistake::DisparateQueryOutputs),
                    }
                }
            }
        }
        mistakes
    }

    pub fn output_files(&self) -> impl Iterator<Item = &Path> {
        self.inner.iter().map(|q| q.output.as_path())
    }

    pub fn get(&self, id: &str) -> Option<&Rc<Query>> {
        self.index.get(id)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Rc<Query>> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

#[cfg(test)]
mod test_util {

    use super::*;

    pub fn strset(xs: Vec<&str>) -> HashSet<String> {
        xs.iter().map(|s| String::from(*s)).collect()
    }

    // Setup queries from &'static str input to be used in tests
    //
    // Argument `data` is a vector of tuple with following type of
    // values (in order):
    //
    //   id: &str, template: &str, conds: Vec<&str>, output: &str
    pub fn setup_queries(data: Vec<(&str, &str, Vec<&str>, &str, Option<&str>)>) -> Queries {
        let mut qs = Queries::new();
        for (id, template, conds, output, name_tag) in data {
            let name_tag = name_tag.map_or_else(
                || NameTag::DeriveFromId(String::from(id)),
                |s| NameTag::Custom(String::from(s)),
            );
            let q = Rc::new(Query {
                id: String::from(id),
                template: PathBuf::from(template),
                conds: strset(conds),
                output: PathBuf::from(output),
                name_tag,
            });
            let idx_key = q.id.clone();
            let idx_val = q.clone();
            qs.inner.push(q);
            qs.index.insert(idx_key, idx_val);
        }
        qs
    }
}

#[cfg(test)]
mod tests {

    use super::test_util::*;
    use super::*;
    use crate::query_template;
    use toml;

    #[test]
    fn test_decode_query() {
        // When all 4 fields are specified
        let table = r#"
id = 'my_query'
template = 'my_query_template.sql.j2'
conds = [ 'foo' ]
output = 'my_query_explicit.sql'
"#
        .parse::<toml::Table>()
        .unwrap();
        let value = toml::Value::Table(table);
        match Query::decode("base", "output", &Layout::OneFileOneQuery, &value) {
            Ok(q) => {
                assert_eq!("my_query", q.id);
                assert_eq!(PathBuf::from("base/my_query_template.sql.j2"), q.template);
                assert_eq!(PathBuf::from("output/my_query_explicit.sql"), q.output);
                assert_eq!(strset(vec!["foo"]), q.conds);
            }
            Err(_) => assert!(false),
        }

        // When conds is not specified
        let table = r#"
id = 'my_query'
template = 'my_query_template.sql.j2'
output = 'my_query_explicit.sql'
"#
        .parse::<toml::Table>()
        .unwrap();
        let value = toml::Value::Table(table);
        match Query::decode("base", "output", &Layout::OneFileOneQuery, &value) {
            Ok(q) => {
                assert_eq!("my_query", q.id);
                assert_eq!(PathBuf::from("base/my_query_template.sql.j2"), q.template);
                assert_eq!(PathBuf::from("output/my_query_explicit.sql"), q.output);
                assert_eq!(HashSet::new(), q.conds);
            }
            Err(_) => assert!(false),
        }

        // When output is not specified
        let table = r#"
id = 'my_query'
template = 'my_query_template.sql.j2'
conds = [ 'foo' ]
"#
        .parse::<toml::Table>()
        .unwrap();
        let value = toml::Value::Table(table);
        match Query::decode("base", "output", &Layout::OneFileOneQuery, &value) {
            Ok(q) => {
                assert_eq!("my_query", q.id);
                assert_eq!(PathBuf::from("base/my_query_template.sql.j2"), q.template);
                assert_eq!(PathBuf::from("output/my_query.sql"), q.output);
                assert_eq!(strset(vec!["foo"]), q.conds);
            }
            Err(_) => assert!(false),
        }

        // When id is not specified
        let table = r#"
template = 'my_query_template.sql.j2'
conds = [ 'foo' ]
output = 'my_query_explicit.sql'
"#
        .parse::<toml::Table>()
        .unwrap();
        let value = toml::Value::Table(table);
        match Query::decode("base", "output", &Layout::OneFileOneQuery, &value) {
            Ok(_) => assert!(false),
            Err(Error::Parsing(msg)) => {
                assert_eq!("Missing 'id' in 'query' entry", msg);
            }
            Err(_) => assert!(false),
        }

        // When template is not specified
        let table = r#"
id = 'my_query'
conds = [ 'foo' ]
output = 'my_query_explicit.sql'
"#
        .parse::<toml::Table>()
        .unwrap();
        let value = toml::Value::Table(table);
        match Query::decode("base", "output", &Layout::OneFileOneQuery, &value) {
            Ok(_) => assert!(false),
            Err(Error::Parsing(msg)) => {
                assert_eq!("Missing 'template' in 'query' entry", msg);
            }
            Err(_) => assert!(false),
        }

        // When `all_conds` is not of correct type
        let table = r#"
id = 'my_query'
template = 'my_query_template.sql.j2'
conds = [ 0 ]
output = 'my_query_explicit.sql'
"#
        .parse::<toml::Table>()
        .unwrap();
        let value = toml::Value::Table(table);
        match Query::decode("base", "output", &Layout::OneFileOneQuery, &value) {
            Ok(_) => assert!(false),
            Err(Error::Parsing(msg)) => {
                assert_eq!(
                    "Value of 'queries[].conds' is expected to be array of strings",
                    msg
                );
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_fallback_output() {
        // When output_layout = OneFileOneQuery
        match fallback_output("foo", "base", &Layout::OneFileOneQuery) {
            Ok(p) => assert_eq!(PathBuf::from("base/foo.sql"), p),
            Err(_) => assert!(false),
        }

        // When output layout = OneFileAllQueries and output_file is
        // specified
        let layout = Layout::OneFileAllQueries(Some(PathBuf::from("base/queries.sql")));
        match fallback_output("foo", "base", &layout) {
            Ok(p) => assert_eq!(PathBuf::from("base/queries.sql"), p),
            Err(_) => assert!(false),
        }

        // When output layout = OneFileAllQueries and output_file is
        // not specified
        let layout = Layout::OneFileAllQueries(None);
        match fallback_output("foo", "base", &layout) {
            Err(Error::Parsing(msg)) => {
                assert_eq!("Either 'queries[].output' or 'query_output_file' must be specified in case of 'one-file-all-queries' layout", msg);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_queries_validate() {
        let qts = query_template::test_util::setup_query_templates(vec![
            (
                "examples/chinook/templates/queries/artists_long_songs.sql.j2",
                vec!["genre", "limit"],
            ),
            (
                "examples/chinook/templates/queries/songs_formats.sql.j2",
                vec!["artist", "file_format", "album_name"],
            ),
        ]);

        // When everything is fine
        let qs_good = setup_queries(vec![
            (
                "artists_long_songs",
                "examples/chinook/templates/queries/artists_long_songs.sql.j2",
                vec![],
                "examples/chinook/output/queries/artists_long_songs.sql",
                None,
            ),
            (
                "songs_formats",
                "examples/chinook/templates/queries/songs_formats.sql.j2",
                vec!["artist", "album_name"],
                "examples/chinook/output/queries/song_formats.sql",
                None,
            ),
        ]);
        let mistakes = qs_good.validate(&qts, &Layout::OneFileOneQuery);
        assert!(mistakes.is_empty());

        // When one of the queries is invalid
        let qs = setup_queries(vec![
            (
                "artists_long_songs",
                "examples/chinook/templates/queries/artists_long_songs.sql.j2",
                vec![],
                "examples/chinook/output/queries/artists_long_songs.sql",
                None,
            ),
            (
                "songs_formats",
                "examples/chinook/templates/queries/undefined.sql.j2",
                vec!["artist", "album_name"],
                "examples/chinook/output/queries/song_formats.sql",
                None,
            ),
        ]);
        let mistakes = qs.validate(&qts, &Layout::OneFileOneQuery);
        assert_eq!(1, mistakes.len());
        match mistakes[0] {
            ManifestMistake::QueryTemplateRefNotFound { query_id, template } => {
                assert_eq!("songs_formats", query_id);
                assert_eq!(
                    "examples/chinook/templates/queries/undefined.sql.j2",
                    template
                );
            }
            _ => assert!(false),
        }

        // When 'queries[].id' are not unique
        let qs = setup_queries(vec![
            (
                "artists_long_songs",
                "examples/chinook/templates/queries/artists_long_songs.sql.j2",
                vec![],
                "examples/chinook/output/queries/artists_long_songs.sql",
                None,
            ),
            (
                "artists_long_songs",
                "examples/chinook/templates/queries/songs_formats.sql.j2",
                vec!["artist", "album_name"],
                "examples/chinook/output/queries/song_formats.sql",
                None,
            ),
        ]);
        let mistakes = qs.validate(&qts, &Layout::OneFileOneQuery);
        assert_eq!(1, mistakes.len());
        match mistakes[0] {
            ManifestMistake::Duplicates { key, value } => {
                assert_eq!("queries[].id", key);
                assert_eq!("artists_long_songs", value);
            }
            _ => assert!(false),
        }

        // When layout = OneFileOneQuery and 'queries[].output' are
        // not unique
        let qs = setup_queries(vec![
            (
                "artists_long_songs",
                "examples/chinook/templates/queries/artists_long_songs.sql.j2",
                vec![],
                "examples/chinook/output/queries/artists_long_songs.sql",
                None,
            ),
            (
                "song_formats",
                "examples/chinook/templates/queries/songs_formats.sql.j2",
                vec!["artist", "album_name"],
                "examples/chinook/output/queries/artists_long_songs.sql",
                None,
            ),
        ]);
        let mistakes = qs.validate(&qts, &Layout::OneFileOneQuery);
        assert_eq!(1, mistakes.len());
        match mistakes[0] {
            ManifestMistake::Duplicates { key, value } => {
                assert_eq!("queries[].output", key);
                assert_eq!(
                    "examples/chinook/output/queries/artists_long_songs.sql",
                    value
                );
            }
            _ => assert!(false),
        }

        // When layout = OneFileAllQueries(None) and 'queries[].output' are
        // not the same
        let layout = Layout::OneFileAllQueries(None);
        let mistakes = qs_good.validate(&qts, &layout);
        assert_eq!(1, mistakes.len());
        match mistakes[0] {
            ManifestMistake::DisparateQueryOutputs => {
                assert!(true);
            }
            _ => assert!(false),
        }

        // When layout = OneFileAllQueries(Some(qof)) and 'queries[].output' are
        // not the same as qof ('query_output_file' in manifest)

        let qs = setup_queries(vec![
            (
                "artists_long_songs",
                "examples/chinook/templates/queries/artists_long_songs.sql.j2",
                vec![],
                "examples/chinook/output/queries.sql",
                None,
            ),
            (
                "songs_formats",
                "examples/chinook/templates/queries/songs_formats.sql.j2",
                vec!["artist", "album_name"],
                "examples/chinook/output/queries/song_formats.sql",
                None,
            ),
        ]);
        let layout =
            Layout::OneFileAllQueries(Some(PathBuf::from("examples/chinook/output/queries.sql")));
        let mistakes = qs.validate(&qts, &layout);
        assert_eq!(1, mistakes.len());
        match mistakes[0] {
            ManifestMistake::InvalidQueryOutput {
                query_id,
                output_path,
            } => {
                assert_eq!("songs_formats", query_id);
                assert_eq!(
                    Path::new("examples/chinook/output/queries/song_formats.sql"),
                    output_path
                );
            }
            _ => assert!(false),
        }
    }
}
