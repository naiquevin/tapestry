use crate::error::Error;
use crate::sql_format::Formatter;
use std::fs;
use std::path::Path;

pub fn ensure_output_dirs<P: AsRef<Path>>(queries_dir: P, tests_dir: P) -> Result<(), Error> {
    fs::create_dir_all(queries_dir.as_ref()).map_err(Error::Io)?;
    fs::create_dir_all(tests_dir.as_ref()).map_err(Error::Io)?;
    Ok(())
}

pub fn write<P: AsRef<Path>>(
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
            Self::Unchanged => "unchanged"
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
            },
            None => {
                if rendered_output.as_bytes() != &contents {
                    Ok(Status::Modified)
                } else {
                    Ok(Status::Unchanged)
                }
            },
        }
    } else {
        Ok(Status::Added)
    }
}
