use crate::error::Error;
use crate::sql_format::Formatter;
use std::fs;
use std::path::Path;

pub fn ensure_output_dirs<P: AsRef<Path>>(queries_dir: P, tests_dir: P) -> Result<(), Error> {
    fs::create_dir_all(queries_dir.as_ref())
        .map_err(Error::Io)?;
    fs::create_dir_all(tests_dir.as_ref())
        .map_err(Error::Io)?;
    Ok(())
}

pub fn write<P: AsRef<Path>>(path: P, formatter: Option<&Formatter>, content: &str) -> Result<(), Error> {
    match formatter {
        Some(f) => {
            let text = f.format(content);
            fs::write(path, text).map_err(Error::Io)?;
        },
        None => {
            fs::write(path, content).map_err(Error::Io)?;
        }
    }
    Ok(())
}
