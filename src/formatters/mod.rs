use crate::{error::Error, toml::SerializableTomlTable};
pub use pg_format::PgFormatter;
use sqlformat_rs::SqlFormat;
use toml::Value;

use self::{config::Configurable, external::ExternalFormatter};

mod config;
mod external;
mod pg_format;
mod sqlformat_rs;
mod util;

/// Enum wrapping over abstractions for various sql formatting tools.
///
/// This indirection is just a provision for plugging in sql
/// formatting tools other than PgFormatter. But at present, only
/// `PgFormatter` is supported.
#[derive(Debug)]
pub enum Formatter {
    PgFormatter(PgFormatter),
    SqlFormatRs(SqlFormat),
}

impl Formatter {
    pub fn decode(value: &Value) -> Result<Option<Self>, Error> {
        match value.as_table() {
            Some(t) => {
                if let Some(v) = t.get("pgFormatter") {
                    return PgFormatter::try_from(v).map(|f| Some(Self::PgFormatter(f)));
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
            Self::SqlFormatRs(f) => f.format(sql),
        }
    }

    pub fn config_toml_table(&self) -> Option<SerializableTomlTable> {
        match self {
            Self::PgFormatter(p) => Some(p.to_toml_table()),
            Self::SqlFormatRs(f) => Some(f.to_toml_table()),
        }
    }

    pub fn discover() -> Option<Self> {
        let pg_format = PgFormatter::new_if_exists().map(Self::PgFormatter);
        if pg_format.is_some() {
            pg_format
        } else {
            // Check for more formatting tools here when support for
            // them is added.
            Some(Self::SqlFormatRs(SqlFormat::default()))
        }
    }
}
