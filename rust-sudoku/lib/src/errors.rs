use std::{
    fmt::{Debug, Display},
    str::Utf8Error,
};

#[derive(Debug, Clone)]
pub struct Error(String);

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&Error> for Error {
    fn from(value: &Error) -> Self {
        value.clone()
    }
}

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Self(format!("{value:?}"))
    }
}
