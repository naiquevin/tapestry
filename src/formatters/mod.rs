use crate::error::Error;
pub use pg_format::PgFormatter;
use toml::Value;

mod pg_format;
mod util;

/// Enum wrapping over abstractions for various sql formatting tools.
///
/// This indirection is just a provision for plugging in sql
/// formatting tools other than PgFormatter. But at present, only
/// `PgFormatter` is supported.
#[derive(Debug)]
pub enum Formatter {
    PgFormatter(PgFormatter),
}

impl Formatter {
    pub fn decode(value: &Value) -> Result<Option<Self>, Error> {
        match value.as_table() {
            Some(t) => {
                if let Some(v) = t.get("pgFormatter") {
                    PgFormatter::try_from(v).map(|f| Some(Self::PgFormatter(f)))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    pub fn format(&self, sql: &str) -> Vec<u8> {
        match self {
            Self::PgFormatter(p) => p.format(sql),
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
