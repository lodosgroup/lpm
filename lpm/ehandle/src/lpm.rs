#[cfg(feature = "sdk")]
use crate::ResultCode;

use std::panic::Location;

pub struct ErrorStack {
    pub file: String,
    pub column: u32,
    pub line: u32,
}

impl std::fmt::Debug for ErrorStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}:{}\"", self.file, self.line)
    }
}

impl ErrorStack {
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

pub struct LpmError<E> {
    pub error_type: E,
    pub chain: Vec<ErrorStack>,
    #[cfg(feature = "sdk")]
    pub result_code: ResultCode,
}

impl<E> std::fmt::Debug for LpmError<E>
where
    E: std::fmt::Debug,
{
    #[cfg(feature = "sdk")]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Result Code: {}({:?}), {:?} From: {:?}",
            self.result_code as u16, self.result_code, self.error_type, self.chain
        )
    }

    #[cfg(not(feature = "sdk"))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} From: {:?}", self.error_type, self.chain)
    }
}

impl<E> LpmError<E> {
    #[track_caller]
    pub fn new(e: E, #[cfg(feature = "sdk")] result_code: ResultCode) -> Self {
        Self {
            error_type: e,
            chain: vec![ErrorStack::new()],
            #[cfg(feature = "sdk")]
            result_code,
        }
    }

    #[track_caller]
    pub fn new_with_traces(
        e: E,
        #[cfg(feature = "sdk")] result_code: ResultCode,
        chain: Vec<ErrorStack>,
    ) -> Self {
        let mut e = Self {
            error_type: e,
            chain,
            #[cfg(feature = "sdk")]
            result_code,
        };
        e.chain.push(ErrorStack::new());

        e
    }
}
