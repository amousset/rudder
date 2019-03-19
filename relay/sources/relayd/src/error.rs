use self::Error::*;
use chrono;
use diesel;
use serde_json;
use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    io, num,
};
use toml;

/// An enum of all error kinds.
#[derive(Debug)]
pub enum Error {
    /// Report parsing
    InvalidReport,
    /// Empty report
    EmptyRunlog,
    /// Internal client error
    Message(String),
    /// Database error
    Database(diesel::result::Error),
    /// Database connection error
    DatabaseConnection(diesel::ConnectionError),
    /// Connection pool error
    Pool(diesel::r2d2::PoolError),
    /// IO error
    Io(io::Error),
    /// TOML error
    Toml(toml::de::Error),
    /// Date error
    DateParsing(chrono::ParseError),
    /// JSON error
    JsonParsing(serde_json::Error),
    /// Parse serial error
    IntegerParsing(num::ParseIntError),
    /// UTF-8 parsing
    Utf8(std::string::FromUtf8Error),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        fmt.write_str(&match *self {
            InvalidReport => "invalid report".to_owned(),
            EmptyRunlog => "agent run log is empty".to_owned(),
            Message(ref message) => message.clone(),
            Database(ref err) => err.to_string(),
            DatabaseConnection(ref err) => err.to_string(),
            Pool(ref err) => err.to_string(),
            Io(ref err) => err.to_string(),
            Toml(ref err) => err.to_string(),
            DateParsing(ref err) => err.to_string(),
            JsonParsing(ref err) => err.to_string(),
            IntegerParsing(ref err) => err.to_string(),
            Utf8(ref err) => err.to_string(),
        })
    }
}

impl StdError for Error {
    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            Database(ref err) => Some(err),
            DatabaseConnection(ref err) => Some(err),
            Pool(ref err) => Some(err),
            Io(ref err) => Some(err),
            Toml(ref err) => Some(err),
            DateParsing(ref err) => Some(err),
            JsonParsing(ref err) => Some(err),
            IntegerParsing(ref err) => Some(err),
            Utf8(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Error {
        Error::Database(err)
    }
}

impl From<diesel::ConnectionError> for Error {
    fn from(err: diesel::ConnectionError) -> Error {
        Error::DatabaseConnection(err)
    }
}

impl From<diesel::r2d2::PoolError> for Error {
    fn from(err: diesel::r2d2::PoolError) -> Error {
        Error::Pool(err)
    }
}

impl From<String> for Error {
    fn from(string: String) -> Error {
        Error::Message(string)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Error {
        Error::Toml(err)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(err: chrono::ParseError) -> Error {
        Error::DateParsing(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonParsing(err)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Error {
        Error::IntegerParsing(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Error {
        Error::Utf8(err)
    }
}
