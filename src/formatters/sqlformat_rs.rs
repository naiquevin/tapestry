use crate::{error::{parse_error, Error}, toml::SerializableTomlTable};
use sqlformat::{FormatOptions, Indent, QueryParams};
use toml::Value;

use super::config::Configurable;

/// Returns default `FormatOptions`
///
/// Note that the `FormatOptions` struct implements the Default trait,
/// but tapestry's default for sqlformat-rs are different from those.
fn default_format_options() -> FormatOptions {
    let mut opts = FormatOptions::default();
    opts.indent = Indent::Spaces(4);
    opts.uppercase = true;
    opts.lines_between_queries = 1;
    opts
}

#[derive(Debug)]
pub struct SqlFormat {
    options: FormatOptions,
}

impl TryFrom<&Value> for SqlFormat {
    type Error = crate::error::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_table() {
            Some(t) => {
                let mut options = default_format_options();
                if let Some(v) = t.get("indent") {
                    if let Some(i) = v.as_integer() {
                        options.indent = Indent::Spaces(i as u8);
                    }
                }
                if let Some(v) = t.get("uppercase") {
                    if let Some(b) = v.as_bool() {
                        options.uppercase = b;
                    }
                }
                if let Some(v) = t.get("lines_between_queries") {
                    if let Some(n) = v.as_integer() {
                        options.lines_between_queries = n as u8;
                    }
                }
                Ok(Self { options })
            }
            None => Err(parse_error!(
                "Value of 'formatter.sqlformat-rs' must be a toml table"
            )),
        }
    }
}

impl Default for SqlFormat {
    fn default() -> Self {
        Self { options: default_format_options() }
    }
}

impl SqlFormat {
    pub fn format(&self, sql: &str) -> Vec<u8> {
        sqlformat::format(sql, &QueryParams::None, self.options).into_bytes()
    }
}

impl Configurable for SqlFormat {
    fn to_toml_table(&self) -> SerializableTomlTable {
        let mut t = SerializableTomlTable::new("formatter.sqlformat-rs");
        t.push_comment("(optional) No. of spaces to indent by");
        t.push_entry_i64("indent", 4);
        t.push_comment("(optional) Use ALL CAPS for reserved keywords");
        t.push_entry_bool("uppercase", true);
        t.push_comment("(optional) No. of line breaks after a query");
        t.push_entry_i64("lines_between_queries", 1);
        t
    }
}
