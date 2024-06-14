use std::path::{Path, PathBuf};
use std::{fs, io};

/// Returns list of files (and not dirs) inside a directory
pub fn ls_files<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, io::Error> {
    let mut result: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            result.push(path);
        }
    }
    Ok(result)
}
