use log::warn;
use toml::Value;

use crate::error::{parse_error, Error};
use crate::sql_format::Formatter;
use crate::tagging::{NameTag, NameTagger};
use crate::toml::decode_pathbuf;
use std::borrow::Cow;
use std::collections::HashSet;
use std::fs;
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

/// Returns status of an output file
///
/// This function compares the `rendered_output` (after formatting if
/// applicable) with the contents of the output file at location
/// `path` without actually writing the file. It returns the
/// appropriate `Status` enum variant as follows,
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
pub fn status<P: AsRef<Path>>(
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
        combined_output.push_str("\n\n");
        let sql = file.tagged_sql(tagger);
        combined_output.push_str(&sql);
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
