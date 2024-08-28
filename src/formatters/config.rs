use crate::toml::SerializableTomlTable;
use std::{fs, io, path::Path};

pub trait Configurable {
    /// Returns a `SerializableTomlTable` that can be used to write
    /// the formatter config as a table in the TOML manifest file.
    fn to_toml_table(&self) -> SerializableTomlTable;

    /// (Optionally) Returns a tuple of path to the config file and
    /// it's contents. This will be called by the provided method
    /// `generate_config_file` to create the config file for the
    /// formatter at the time of project initialization
    fn config_file(&self) -> Option<(&Path, &'static str)>;

    /// Creates the config file for the formatter at the time of
    /// project initialization
    fn generate_config_file(&self, dir: &Path) -> io::Result<()> {
        match self.config_file() {
            Some((path, contents)) => {
                let conf_path = dir.join(path);
                let conf_subdir = path.parent().unwrap();
                if conf_subdir != Path::new(".") && conf_subdir != Path::new("") {
                    fs::create_dir(conf_path.parent().unwrap())?;
                }
                fs::write(conf_path, contents)?;
                Ok(())
            }
            None => Ok(()),
        }
    }
}
