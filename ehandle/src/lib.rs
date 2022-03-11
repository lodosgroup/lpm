#![forbid(unsafe_code)]
#![feature(io_error_more, io_error_uncategorized)]

use std::error;

#[derive(Debug)]
pub struct RuntimeError {
    pub kind: String,
    pub reason: String,
}

impl From<RuntimeError> for Box<dyn error::Error> {
    fn from(error: RuntimeError) -> Self {
        error.into()
    }
}

impl From<Box<dyn std::error::Error>> for RuntimeError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        RuntimeError {
            kind: "TODO".to_string(), // error.source().unwrap().to_string(),
            reason: error.to_string(),
        }
    }
}

mod io;
pub mod package;

