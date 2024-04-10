use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Toml(toml::de::Error),
    Parsing(String),
    UndefinedQuery(String),
    UndefinedQueryTemplate(String),
    UndefinedTestTemplate(String),
    MiniJinja(minijinja::Error),
}

macro_rules! parse_error {
    ($($x:tt)*) => {{
        let msg = std::fmt::format(format_args!($($x)*));
        Error::Parsing(msg)
    }}
}

pub(crate) use parse_error;
