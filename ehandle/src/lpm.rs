use std::{io, panic::Location};

use crate::{
    db::{MigrationError, SqlError},
    pkg::PackageError,
};

#[derive(Debug)]
pub struct ErrorStackTrace {
    pub file: String,
    pub column: u32,
    pub line: u32,
}

impl ErrorStackTrace {
    #[track_caller]
    fn new() -> Self {
        let caller = Location::caller();
        Self {
            file: caller.file().to_string(),
            column: caller.column(),
            line: caller.line(),
        }
    }
}

#[derive(Debug)]
pub struct LpmError<E> {
    pub error_type: E,
    pub error_stack: Vec<ErrorStackTrace>,
}

impl<E> LpmError<E> {
    #[track_caller]
    pub fn new(e: E) -> Self {
        Self {
            error_type: e,
            error_stack: vec![ErrorStackTrace::new()],
        }
    }

    #[track_caller]
    pub fn new_with_traces(e: E, error_stack: Vec<ErrorStackTrace>) -> Self {
        let mut e = Self {
            error_type: e,
            error_stack,
        };
        e.error_stack.push(ErrorStackTrace::new());

        e
    }
}
