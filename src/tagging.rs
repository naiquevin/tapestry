use regex::Regex;
use std::borrow::Cow;
use std::fmt;
use toml::Value;

use crate::error::{parse_error, Error};

#[derive(Debug)]
pub enum NameTag {
    DeriveFromId(String),
    Custom(String),
}

#[derive(Debug)]
pub enum NameTagStyle {
    SnakeCase,
    KebabCase,
    Exact,
}

impl NameTagStyle {
    fn decode(value: &Value) -> Result<Self, Error> {
        match value.as_str() {
            Some(s) => {
                // @NOTE the case used options is not consistent! They
                // are spelt autologically
                match s {
                    "snake_case" => Ok(Self::SnakeCase),
                    "kebab-case" => Ok(Self::KebabCase),
                    "exact" => Ok(Self::Exact),
                    _ => Err(parse_error!("Invalid value for 'name_tagger.style': {s}")),
                }
            }
            None => Err(parse_error!(
                "Value of 'name_tagger.style' expected to be a string"
            )),
        }
    }

    // Constructs a tag from the id as per the `NameTagStyle`. Any
    // non-alphanumeric char will be replaced by either hyphen
    // (kebab-case) or underscore (snake_case).
    pub fn make_tag<'a>(&self, id: &'a str) -> Cow<'a, str> {
        // @TODO: Can this regex pattern be defined globally?
        let re = Regex::new(r"_|-|@|\+|&|\*").unwrap();
        match self {
            Self::SnakeCase => re.replace_all(id, "_"),
            Self::KebabCase => re.replace_all(id, "-"),
            Self::Exact => Cow::from(id),
        }
    }
}

impl fmt::Display for NameTagStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NameTagStyle::SnakeCase => write!(f, "snake_case"),
            NameTagStyle::KebabCase => write!(f, "kebab-case"),
            NameTagStyle::Exact => write!(f, "exact"),
        }
    }
}

// @TODO: It's sufficient to check that the first line is the name
// tag. It can be assumed that the docstrings will follow the name
// tag.
fn has_name_tag(sql: &str) -> bool {
    let pattern = Regex::new(r"--\s*name:\s*\w+").unwrap();
    let pre_comments = sql.lines().take_while(|s| s.starts_with("--"));
    for line in pre_comments {
        if pattern.is_match(line) {
            return true;
        }
    }
    false
}

#[derive(Debug)]
pub struct NameTagger {
    pub style: NameTagStyle,
}

impl NameTagger {
    pub fn decode(value: &Value) -> Result<Option<Self>, Error> {
        match value.as_table() {
            Some(t) => {
                let style = t
                    .get("style")
                    .ok_or(parse_error!("Key 'name_tagger.style' is missing"))
                    .map(NameTagStyle::decode)??;
                Ok(Some(Self { style }))
            }
            None => Ok(None),
        }
    }

    pub fn make_name_tag(&self, name_tag: &NameTag) -> String {
        let tag = match name_tag {
            NameTag::DeriveFromId(id) => self.style.make_tag(id),
            NameTag::Custom(s) => Cow::from(s),
        };
        format!("-- name: {tag}")
    }

    // Prepends `sql` with a comment line containing the name tag if one
    // doesn't already exist
    //
    // Note that this function also trims any leading blank lines
    pub fn ensure_name_tag<'a>(&self, sql: &'a str, name_tag: &'a NameTag) -> Cow<'a, str> {
        let sql = sql.trim_start();
        if has_name_tag(sql) {
            Cow::from(sql)
        } else {
            let mut result = self.make_name_tag(name_tag);
            result.push('\n');
            result.push_str(sql);
            Cow::from(result)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_ensure_name_tag() {
        let tag_style = NameTagStyle::SnakeCase;
        let tagger = NameTagger { style: tag_style };

        // When name tag doesn't exist
        let sql = "SELECT 1;";
        let name_tag = NameTag::DeriveFromId("simple-query".to_owned());
        let expected = r#"-- name: simple_query
SELECT 1;"#;
        assert_eq!(expected, tagger.ensure_name_tag(sql, &name_tag));

        // When name tag exists (along with whitespace before the
        // pre_comments and additional comments/documentation)
        let sql = r#"
-- name: simple_query
-- Fetch static value one
SELECT 1;

"#;
        let expected = r#"-- name: simple_query
-- Fetch static value one
SELECT 1;

"#;
        assert_eq!(expected, tagger.ensure_name_tag(sql, &name_tag));

        // When a comment similar to the name tag is found somewhere
        // in the middle of the sql
        let sql = r#"
SELECT
    *
FROM
    employees
WHERE
-- name: this line is not considered a name tag
    email = 'email'
    AND department = 'department';
"#;
        let name_tag = NameTag::DeriveFromId("find_employee@email+dept".to_owned());
        let expected = r#"-- name: find_employee_email_dept
SELECT
    *
FROM
    employees
WHERE
-- name: this line is not considered a name tag
    email = 'email'
    AND department = 'department';
"#;
        assert_eq!(expected, tagger.ensure_name_tag(sql, &name_tag));
    }
}
