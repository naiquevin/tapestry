use crate::error::{parse_error, Error};
use std::convert::TryFrom;
use toml::Value;

#[derive(Debug)]
pub enum Placeholder {
    PosArgs,
    Variables,
}

impl TryFrom<&Value> for Placeholder {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_str() {
            Some(s) => {
                if s == "posargs" {
                    Ok(Self::PosArgs)
                } else if s == "variables" {
                    Ok(Self::Variables)
                } else {
                    Err(parse_error!("Invalid placeholder: '{}", s))
                }
            }
            None => Err(parse_error!("Value of key 'placeholder' must be a string")),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use toml::Table;

    #[test]
    fn test_placeholder_try_from() {
        let t = "placeholder = 'posargs'".parse::<Table>().unwrap();
        let p = Placeholder::try_from(&t["placeholder"]);
        assert!(p.is_ok());
        match p.unwrap() {
            Placeholder::PosArgs => assert!(true),
            _ => assert!(false),
        }

        let t = "placeholder = 'variables'".parse::<Table>().unwrap();
        let p = Placeholder::try_from(&t["placeholder"]);
        assert!(p.is_ok());
        match p.unwrap() {
            Placeholder::Variables => assert!(true),
            _ => assert!(false),
        }

        let t = "placeholder = 'question-marks'".parse::<Table>().unwrap();
        let p = Placeholder::try_from(&t["placeholder"]);
        assert!(p.is_err());
    }
}
