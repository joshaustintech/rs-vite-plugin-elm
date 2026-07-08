pub mod assets;
pub mod compile;
pub mod deps;
pub mod elm_make;
pub mod esm;
pub mod hmr;
pub mod import_id;
pub mod options;

#[cfg(test)]
pub mod speed;

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(String);

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub use compile::{CompileOutput, CompileRequest, compile};

#[cfg(test)]
mod tests;
