use crate::error::{parse_error, Error};
use sqlformat::{FormatOptions, Indent, QueryParams};
use toml::Value;

#[derive(Debug)]
pub struct SqlFormat {
    options: FormatOptions,
}

impl TryFrom<&Value> for SqlFormat {
    type Error = crate::error::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_table() {
            Some(t) => {
                let mut options = FormatOptions::default();
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

impl SqlFormat {
    pub fn format(&self, sql: &str) -> Vec<u8> {
        sqlformat::format(sql, &QueryParams::None, self.options).into_bytes()
    }
}
