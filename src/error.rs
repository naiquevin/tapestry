use std::fmt::{self, Display};
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
    Scaffolding(String),
    ManifestNotFound,
    Cli(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ManifestNotFound => {
                write!(f, "Manifest file 'tapestry.toml' not found\nTip: Ensure you're inside the correct directory")
            }
            Self::Cli(msg) => write!(f, "Command error: {msg}"),
            Self::Scaffolding(msg) => {
                write!(f, "Error initializing new tapestry project\nReason: {msg}")
            }
            Self::Io(e) => write!(f, "I/O Error: {e:?}"),
            Self::Toml(e) => write!(f, "TOML Error: {e:?}"),
            Self::Parsing(msg) => write!(f, "Error parsing manifest file: {msg}"),
            Self::UndefinedQuery(id) => write!(f, "Lookup for query failed: id={id}"),
            Self::UndefinedQueryTemplate(path) => {
                write!(f, "Lookup for query template failed: path={path}")
            }
            Self::UndefinedTestTemplate(path) => {
                write!(f, "Lookup for test template failed: path={path}")
            }
            Self::MiniJinja(e) => write!(f, "MiniJinja Error: {e:?}"),
        }
    }
}

macro_rules! parse_error {
    ($($x:tt)*) => {{
        let msg = std::fmt::format(format_args!($($x)*));
        Error::Parsing(msg)
    }}
}

pub(crate) use parse_error;
