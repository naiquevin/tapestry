use log::warn;
use toml::Value;

use crate::error::{parse_error, Error};
use crate::formatters::Formatter;
use crate::metadata::Metadata;
use crate::query::{Queries, Query};
use crate::tagging::{NameTag, NameTagger};
use crate::toml::decode_pathbuf;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

pub fn ensure_output_dirs<P: AsRef<Path>>(queries_dir: P, tests_dir: P) -> Result<(), Error> {
    fs::create_dir_all(queries_dir.as_ref()).map_err(Error::Io)?;
    fs::create_dir_all(tests_dir.as_ref()).map_err(Error::Io)?;
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Added,
    Modified,
    Unchanged,
}

impl Status {
    pub fn label(&self) -> &str {
        match &self {
            Self::Added => "added",
            Self::Modified => "modified",
            Self::Unchanged => "unchanged",
        }
    }
}

fn ensure_trailing_newline(sql: &str) -> Cow<'_, str> {
    if !sql.ends_with('\n') {
        let mut s = sql.to_string();
        s.push('\n');
        Cow::Owned(s)
    } else {
        Cow::Borrowed(sql)
    }
}

/// Returns status of a query output file without modifying it
///
/// This function compares the `rendered_output` (after formatting if
/// applicable) with the contents of the output file obtained using
/// the `QueryOutputReader`. It returns the appropriate `Status` enum
/// variant as follows,
///
///   Status::Added - if the output file doesn't already exist i.e. it
///   would get added upon calling the `render` command
///
///   Status::Modified - if the `rendered_output` (after formatting if
///   applicable) is different from the contents of the existing
///   output file i.e. the file would get modified upon calling the
///   `render` command.
///
///   Status::Unchanged - if the `rendered_output` (after formatting
///   if applicable) is exactly the same as that contents of the
///   existing output file
///
/// Returns `Error::Io` error if an error is encountered while reading
/// the output file.
///
pub fn query_status(
    query: &Query,
    reader: &QueryOutputReader,
    formatter: Option<&Formatter>,
    rendered_output: &str,
) -> Result<Status, Error> {
    let exists = reader.exists(&query.id)?;
    if exists {
        let contents = reader.read(&query.id)?;
        // @NOTE: The code duplication below is intentional. It
        // prevents unnecessary conversion of byte array into vec
        match formatter {
            Some(f) => {
                if f.format(rendered_output) != contents {
                    Ok(Status::Modified)
                } else {
                    Ok(Status::Unchanged)
                }
            }
            None => {
                let output = ensure_trailing_newline(rendered_output);
                if output.as_bytes() != contents {
                    Ok(Status::Modified)
                } else {
                    Ok(Status::Unchanged)
                }
            }
        }
    } else {
        Ok(Status::Added)
    }
}

/// Returns status of a test output file without modifying it
///
/// This function compares the `rendered_output` (after formatting if
/// applicable) with the contents of the output file at location
/// `path`. It returns the appropriate `Status` enum variant as
/// follows,
///
///   Status::Added - if the output file doesn't already exist i.e. it
///   would get added upon calling the `render` command
///
///   Status::Modified - if the `rendered_output` (after formatting if
///   applicable) is different from the contents of the existing
///   output file i.e. the file would get modified upon calling the
///   `render` command.
///
///   Status::Unchanged - if the `rendered_output` (after formatting
///   if applicable) is exactly the same as that contents of the
///   existing output file
///
/// Returns `Error::Io` error if an error is encountered while reading
/// the output file.
///
pub fn testfile_status<P: AsRef<Path>>(
    path: P,
    formatter: Option<&Formatter>,
    rendered_output: &str,
) -> Result<Status, Error> {
    let exists = path.as_ref().try_exists().map_err(Error::Io)?;
    if exists {
        let contents = fs::read(path).map_err(Error::Io)?;
        // @NOTE: The code duplication below is intentional. It
        // prevents unnecessary conversion of byte array into vec
        match formatter {
            Some(f) => {
                if f.format(rendered_output) != contents {
                    Ok(Status::Modified)
                } else {
                    Ok(Status::Unchanged)
                }
            }
            None => {
                if rendered_output.as_bytes() != contents {
                    Ok(Status::Modified)
                } else {
                    Ok(Status::Unchanged)
                }
            }
        }
    } else {
        Ok(Status::Added)
    }
}

#[derive(Debug)]
pub enum Layout {
    OneFileOneQuery,
    OneFileAllQueries(Option<PathBuf>),
}

impl Layout {
    pub fn default() -> Self {
        Self::OneFileOneQuery
    }

    pub fn decode<P: AsRef<Path>>(
        value: &Value,
        output_file: Option<&Value>,
        output_base_dir: P,
    ) -> Result<Self, Error> {
        match value.as_str() {
            Some(s) => {
                if s == "one-file-one-query" {
                    // @TODO: warn if output file is specified
                    Ok(Self::OneFileOneQuery)
                } else if s == "one-file-all-queries" {
                    let filepath = match output_file {
                        Some(v) => Some(decode_pathbuf(
                            v,
                            Some(output_base_dir.as_ref()),
                            "queries_output_file",
                        )?),
                        None => None,
                    };
                    Ok(Self::OneFileAllQueries(filepath))
                } else {
                    Err(parse_error!(
                        "Invalid value for 'query_output_layout': '{s}'"
                    ))
                }
            }
            None => {
                warn!("Invalid value for 'query_output_layout' in the manifest. Using 'one-file-one-query' as the default.");
                Ok(Self::OneFileOneQuery)
            }
        }
    }
}

// Struct for representing output files that need to written
pub struct SqlToWrite<'a> {
    pub path: &'a Path,
    pub sql: String,
    pub name_tag: Option<&'a NameTag>,
}

impl<'a> SqlToWrite<'a> {
    // Returns tagged sql if applicable
    //
    // Tagging is done only if both the conditions are satisfied:
    //   1. `tagger` is not None
    //   2. `self.name_tag` is not None
    fn tagged_sql(&'a self, tagger: Option<&NameTagger>) -> Cow<'a, str> {
        match tagger {
            Some(t) => match self.name_tag {
                Some(nt) => t.ensure_name_tag(&self.sql, nt),
                None => Cow::from(&self.sql),
            },
            None => Cow::from(&self.sql),
        }
    }
}

fn write<P: AsRef<Path>>(
    path: P,
    formatter: Option<&Formatter>,
    content: &str,
) -> Result<(), Error> {
    match formatter {
        Some(f) => {
            let text = f.format(content);
            fs::write(path, text).map_err(Error::Io)?;
        }
        None => {
            fs::write(path, content).map_err(Error::Io)?;
        }
    }
    Ok(())
}

// Combines file contents and writes to a single file
//
// # Panics!
// This function is only supposed to be called when all paths in the
// passed Vec<FileToWrite> are equal i.e. all the content is to be
// written to the same file (Layout = OneFileAllQueries). If this
// condition is not satisfied, this functions panics.
//
// @TODO: This function currently concatenates Strings in memory and
// then writes in a single call. A more memory efficient approach
// would be to keep the file open and write each query to it one by
// one
pub fn write_combined(
    files: &Vec<SqlToWrite>,
    formatter: Option<&Formatter>,
    tagger: Option<&NameTagger>,
) -> Result<(), Error> {
    let mut combined_output = String::new();
    let mut paths = Vec::with_capacity(files.len());
    for file in files {
        let sql = file.tagged_sql(tagger);
        combined_output.push_str(&sql);
        combined_output.push('\n');
        combined_output.push('\n');
        paths.push(file.path);
    }
    let mut path_set: HashSet<&Path> = HashSet::from_iter(paths);
    if path_set.len() > 1 {
        panic!("write_combined function called with disparate file paths. Please report this bug");
    }
    let filepath = path_set.drain().next().unwrap();
    write(filepath, formatter, &combined_output)
}

// Writes file contents to separate files in a loop
pub fn write_separately(
    files: &Vec<SqlToWrite>,
    formatter: Option<&Formatter>,
    tagger: Option<&NameTagger>,
) -> Result<(), Error> {
    for file in files {
        let sql = file.tagged_sql(tagger);
        write(file.path, formatter, &sql)?;
    }
    Ok(())
}

// @TODO: Add tests
fn parse_combined_sql<'a>(
    filepath: &Path,
    tagger: &NameTagger,
    formatter: Option<&Formatter>,
    queries: &'a Queries,
) -> Result<HashMap<&'a str, String>, Error> {
    let tags_to_ids = queries
        .iter()
        .map(|q| {
            let tag = tagger.make_name_tag(&q.name_tag);
            (tag, q.id.as_str())
        })
        .collect::<HashMap<String, &str>>();
    let mut curr_id: Option<&str> = None;
    let mut result: HashMap<&str, String> = HashMap::with_capacity(tags_to_ids.len());

    let file = File::open(filepath).map_err(Error::Io)?;

    // @NOTE: `map_while(Result::ok)` is the equivalent of flatten
    for line in io::BufReader::new(file).lines().map_while(Result::ok) {
        match tags_to_ids.get(&line) {
            Some(id) => {
                curr_id = Some(id);
                result.insert(id, line);
            }
            None => {
                if curr_id.is_none() && line.is_empty() {
                    continue;
                }
                let key =
                    curr_id.ok_or(Error::QueryOutputParsing(filepath.display().to_string()))?;
                if let Some(qlines) = result.get_mut(key) {
                    qlines.push('\n');
                    qlines.push_str(line.as_str());
                } else {
                    warn!("Query id not identified when parsing output file. Discarding line");
                }
            }
        }
    }

    if formatter.is_some() {
        // @NOTE: If formatter is specified, add extra trailing
        // newline for each query. Reason for doing this: The
        // formatting operation adds a trailing newline to each
        // sql. This means when every query is individually formatted,
        // it will be formatted with a trailing newline. But the same
        // doesn't happen when all queries are in a single file. This
        // workaround is to just for it so that comparison can be done
        // more cleanly.
        result.iter_mut().for_each(|(_, val)| {
            val.push('\n');
        });
    }

    // Convert acc to cache hashmap
    Ok(result)
}

/// Abstraction for reading query output files based on the layout
/// i.e. if layout = OneFileOneQuery, then it will be read from
/// individual files, otherwise if layout = OneFileAllQueries, then
/// the individual queries will be first parsed by reading the single
/// output and cached
pub struct QueryOutputReader<'a> {
    metadata: &'a Metadata,
    parsed_query_store: Option<HashMap<&'a str, String>>,
}

impl<'a> QueryOutputReader<'a> {
    pub fn new(metadata: &'a Metadata) -> Result<Self, Error> {
        // @TODO: Instead of eagerly loading data, we could have used
        // OnceCell here. But currently, the
        // `OnceCell.get_or_try_init` is a nightly only feature.
        let pqs = match metadata.query_output_layout {
            Layout::OneFileOneQuery => None,
            Layout::OneFileAllQueries(_) => {
                // Unwrap is acceptable as None value is expected only
                // if the layout = OneFileOneQuery
                let filepath = metadata.combined_output_file()?.unwrap();
                let tagger = metadata.name_tagger.as_ref().ok_or(Error::Layout(
                    "name_tagger is required when layout = one-file-all-queries".to_string(),
                ))?;
                Some(parse_combined_sql(
                    filepath,
                    tagger,
                    metadata.formatter.as_ref(),
                    &metadata.queries,
                )?)
            }
        };
        Ok(Self {
            metadata,
            parsed_query_store: pqs,
        })
    }

    pub fn read(&self, query_id: &str) -> Result<Vec<u8>, Error> {
        let query = self
            .metadata
            .queries
            .get(query_id)
            .ok_or(Error::UndefinedQuery(query_id.to_owned()))?;
        match self.metadata.query_output_layout {
            Layout::OneFileOneQuery => fs::read(&query.output).map_err(Error::Io),
            Layout::OneFileAllQueries(_) => {
                let query_store =
                    self.parsed_query_store
                        .as_ref()
                        .ok_or(Error::QueryOutputParsing(
                            "QueryOutputReader.parsed_query_store field not initialized"
                                .to_string(),
                        ))?;
                query_store
                    .get(query_id)
                    .map(|s| s.clone().into_bytes())
                    .ok_or(Error::QueryOutputParsing(format!(
                        "Query not found in combined SQL file: {}",
                        query.output.display(),
                    )))
            }
        }
    }

    pub fn exists(&self, query_id: &str) -> Result<bool, Error> {
        let query = self
            .metadata
            .queries
            .get(query_id)
            .ok_or(Error::UndefinedQuery(query_id.to_owned()))?;
        match self.metadata.query_output_layout {
            Layout::OneFileOneQuery => query.output.try_exists().map_err(Error::Io),
            Layout::OneFileAllQueries(_) => {
                let query_store =
                    self.parsed_query_store
                        .as_ref()
                        .ok_or(Error::QueryOutputParsing(
                            "QueryOutputReader.parsed_query_store field not initialized"
                                .to_string(),
                        ))?;
                Ok(query_store.contains_key(&query_id))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml::Table;

    #[test]
    fn test_layout_decode() {
        // When layout = 'one-file-one-query' AND output file is not
        // specified
        let manifest = "query_output_layout = 'one-file-one-query'";
        let table: Table = manifest.parse().unwrap();
        let layout = Layout::decode(
            table.get("query_output_layout").unwrap(),
            table.get("query_output_file"),
            "base",
        );
        match layout {
            Ok(Layout::OneFileOneQuery) => assert!(true),
            _ => assert!(false),
        }

        // When layout = 'one-file-one-query' AND output file is
        // specified, the output file is ignored
        let manifest = r#"
query_output_layout = 'one-file-one-query'
query_output_file = 'queries.sql'
"#;
        let table: Table = manifest.parse().unwrap();
        let layout = Layout::decode(
            table.get("query_output_layout").unwrap(),
            table.get("query_output_file"),
            "base",
        );
        match layout {
            Ok(Layout::OneFileOneQuery) => assert!(true),
            _ => assert!(false),
        }

        // When layout = 'one-file-all-queries' AND output file is not
        // specified
        let manifest = "query_output_layout = 'one-file-all-queries'";
        let table: Table = manifest.parse().unwrap();
        let layout = Layout::decode(
            table.get("query_output_layout").unwrap(),
            table.get("query_output_file"),
            "base",
        );
        match layout {
            Ok(Layout::OneFileAllQueries(p)) => {
                assert!(true);
                assert_eq!(None, p);
            }
            _ => assert!(false),
        };

        // When layout = 'one-file-all-queries' AND output file is
        // specified
        let manifest = r#"
query_output_layout = 'one-file-all-queries'
query_output_file = 'queries.sql'
"#;
        let table: Table = manifest.parse().unwrap();
        let layout = Layout::decode(
            table.get("query_output_layout").unwrap(),
            table.get("query_output_file"),
            "base",
        );
        match layout {
            Ok(Layout::OneFileAllQueries(p)) => {
                assert!(true);
                assert_eq!(Some(PathBuf::from("base/queries.sql")), p);
            }
            _ => assert!(false),
        };
    }
}
