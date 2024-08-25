use crate::toml::SerializableTomlTable;

pub trait Configurable {
    /// Returns a `SerializableTomlTable` that can be used to write
    /// the formatter config as a table in the TOML config file.
    fn to_toml_table(&self) -> SerializableTomlTable;
}
