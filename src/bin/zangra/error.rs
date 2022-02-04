use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter};
use std::result;

use serenity::Error as SerenityError;
use serde_json::Error as JsonError;

pub type Result<T> = result::Result<T, Error>;

pub struct ZangraError {
    message: String,
}

impl ZangraError {
    pub fn new<S: Into<String>>(message: S) -> ZangraError{
        ZangraError {
            message: message.into()
        }
    }
}

impl Display for ZangraError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_str())

    }
}

impl Debug for ZangraError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_str())
    }
}

impl StdError for ZangraError {
    fn description(&self) -> &str {
        todo!()
    }
}

#[derive(Debug)]
pub enum Error {
    Json(JsonError),
    Serenity(SerenityError),
    Zangra(ZangraError),
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Self {
        Error::Json(e)
    }
}

impl From<SerenityError> for Error {
    fn from(e: SerenityError) -> Self {
        Error::Serenity(e)
    }
}

impl From<ZangraError> for Error {
    fn from(e: ZangraError) -> Self {
        Error::Zangra(e)
    }
}



