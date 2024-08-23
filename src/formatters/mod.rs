use crate::error::Error;
pub use pg_format::PgFormatter;
use sqlformat_rs::SqlFormat;
use toml::Value;

use self::external::ExternalFormatter;

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

    pub fn discover() -> Option<Self> {
        let pg_format = PgFormatter::new_if_exists().map(Self::PgFormatter);
        if pg_format.is_some() {
            pg_format
        } else {
            // Check for more formatting tools here when support for
            // them is added.
            None
        }
    }
}
