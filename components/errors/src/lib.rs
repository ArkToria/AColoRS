use std::error::Error as StdError;

#[derive(Debug)]
pub enum ErrorKind {}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub source: Option<Box<dyn StdError + Send + Sync>>,
}
pub type Result<T> = ::std::result::Result<T, Error>;
