use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Toml(toml::de::Error),
    Parsing(String),
}

macro_rules! parse_error {
    ($($x:tt)*) => {{
        let msg = std::fmt::format(format_args!($($x)*));
        Error::Parsing(msg)
    }}
}

pub(crate) use parse_error;
