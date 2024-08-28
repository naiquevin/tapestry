use self::config::Configurable;
use self::external::ExternalFormatter;
use self::sqlfluff::SqlFluff;
use crate::{error::Error, toml::SerializableTomlTable};
pub use pg_format::PgFormatter;
use sqlformat_rs::SqlFormat;
use std::path::Path;
use toml::Value;

mod config;
mod external;
mod pg_format;
mod sqlfluff;
mod sqlformat_rs;
mod util;

/// Enum wrapping over abstractions for various sql formatting tools.
///
/// This indirection is just a provision for plugging in sql
/// formatting tools other than PgFormatter. But at present, only
/// `PgFormatter` is supported.
#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum Formatter {
    PgFormatter(PgFormatter),
    SqlFormatRs(SqlFormat),
    SqlFluff(SqlFluff),
}

impl Formatter {
    pub fn decode(value: &Value) -> Result<Option<Self>, Error> {
        match value.as_table() {
            Some(t) => {
                if let Some(v) = t.get("pgFormatter") {
                    return PgFormatter::try_from(v).map(|f| Some(Self::PgFormatter(f)));
                }
                if let Some(v) = t.get("sqlfluff") {
                    return SqlFluff::try_from(v).map(|f| Some(Self::SqlFluff(f)));
                }
                if let Some(v) = t.get("sqlformat-rs") {
                    return SqlFormat::try_from(v).map(|f| Some(Self::SqlFormatRs(f)));
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn format(&self, sql: &str) -> Vec<u8> {
        match self {
            Self::PgFormatter(p) => p.format(sql),
            Self::SqlFluff(f) => f.format(sql),
            Self::SqlFormatRs(f) => f.format(sql),
        }
    }

    pub fn config_toml_table(&self) -> Option<SerializableTomlTable> {
        match self {
            Self::PgFormatter(p) => Some(p.to_toml_table()),
            Self::SqlFluff(f) => Some(f.to_toml_table()),
            Self::SqlFormatRs(f) => Some(f.to_toml_table()),
        }
    }

    pub fn executable(&self) -> Option<&Path> {
        match self {
            Self::PgFormatter(p) => Some(p.executable()),
            Self::SqlFluff(f) => Some(f.executable()),
            Self::SqlFormatRs(_) => None,
        }
    }

    pub fn generate_config_file(&self, dir: &Path) -> Result<(), Error> {
        let res = match self {
            Self::PgFormatter(p) => p.generate_config_file(dir),
            Self::SqlFormatRs(f) => f.generate_config_file(dir),
            Self::SqlFluff(f) => f.generate_config_file(dir),
        };
        res.map_err(Error::Io)
    }
}

/// Returns an ordered vec of formatters discovered on the system.
///
/// It includes the builtin sqlformat as well, and it's the first item
/// in the result
pub fn discover_available_formatters() -> Vec<Formatter> {
    let mut formatters = vec![];
    // @NOTE: The following order needs to be maintained

    // 1. builtin formatter sqlformat
    formatters.push(Formatter::SqlFormatRs(SqlFormat::default()));

    // 2. pgFormatter
    if let Some(pgf) = PgFormatter::discover() {
        formatters.push(Formatter::PgFormatter(pgf));
    }

    // 3. sqlfluff
    if let Some(sf) = SqlFluff::discover() {
        formatters.push(Formatter::SqlFluff(sf));
    }
    formatters
}
