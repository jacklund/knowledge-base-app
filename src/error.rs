use std::fmt;
use surrealdb;

#[derive(Debug)]
pub enum Error {
    DbError(surrealdb::Error),
    KbError(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DbError(e) => write!(f, "{}", e),
            Error::KbError(e) => write!(f, "Error: {}", e),
        }
    }
}

impl From<surrealdb::Error> for Error {
    fn from(e: surrealdb::Error) -> Self {
        Self::DbError(e)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::KbError(s.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::KbError(s)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
