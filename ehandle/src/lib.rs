#![forbid(unsafe_code)]
#![feature(io_error_more, io_error_uncategorized)]

use std::error;

#[non_exhaustive]
#[derive(Debug)]
pub enum RuntimeErrorKind {
    UnsupportedPlatform,
}

impl RuntimeErrorKind {
    pub fn as_str(&self) -> &str {
        match self {
            RuntimeErrorKind::UnsupportedPlatform => "UnsupportedPlatform",
        }
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub kind: String,
    pub reason: String,
}

impl RuntimeError {
    pub fn new(kind: RuntimeErrorKind) -> Self {
        match kind {
            RuntimeErrorKind::UnsupportedPlatform => RuntimeError {
                kind: kind.as_str().to_string(),
                reason: "LodPM can only work on Linux based platforms.".to_string(),
            },
        }
    }
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

pub mod db;
mod io;
pub mod package;
