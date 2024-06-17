use std::path::{Path, PathBuf};
use std::{fs, io};

/// Checks whether a file is a backup file
///
/// In emacs, the backup files are suffixed with tilde `~`. Returns
/// false if it's not able to find the extension or convert OsString
/// into &str
fn is_backup_file<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref()
        .extension()
        .map_or(false, |e| e.to_str().map_or(false, |s| s.ends_with('~')))
}

/// Returns list of files (and not dirs) inside a directory
pub fn ls_files<P: AsRef<Path>>(path: P, include_backups: bool) -> Result<Vec<PathBuf>, io::Error> {
    let mut result: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && (include_backups || !is_backup_file(&path)) {
            result.push(path);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_is_backup_file() {
        assert!(is_backup_file("hello.sql~"));
        assert!(!is_backup_file("hello.sql"));
    }
}
