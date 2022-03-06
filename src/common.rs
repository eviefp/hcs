use iso8061_timestamp::Timestamp;
use serde_derive::Deserialize;
use std::error::Error;
use std::fmt;

/****************************************************************************************
* Error types
****************************************************************************************/

#[derive(Debug)]
pub enum HcsError {
    InsertDataError {},
    TodayError {},
    TomorrowError {},
    NextError {},
    CalError {},
    ImportMissingKey {},
    MissingStart {},
}

impl fmt::Display for HcsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InsertError")
    }
}

impl Error for HcsError {}

// This is the name that comes from the GraphQL schema and I don't want to manually change it.
// It's required by the macro to generate code.
#[allow(non_camel_case_types)]
pub type timestamptz = Timestamp;

/****************************************************************************************
* TOML config
****************************************************************************************/

#[derive(Deserialize)]
pub struct Config {
    pub hasura: Hasura,
    pub import: Vec<Import>,
}

#[derive(Deserialize)]
pub struct Hasura {
    pub url: String,
    pub admin_key: String,
}

#[derive(Deserialize, Debug)]
pub struct Import {
    pub name: String,
    pub url: String,
}
