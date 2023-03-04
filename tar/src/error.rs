use std::borrow::Cow;
use std::error;
use std::fmt;
use std::io::{self, Error};

#[macro_export]
macro_rules! err {
    ($err: expr) => {
        std::io::Error::new(std::io::ErrorKind::Other, $err)
    };
}

#[derive(Debug)]
pub struct TarError {
    desc: Cow<'static, str>,
    io: io::Error,
}

impl TarError {
    pub fn new(desc: impl Into<Cow<'static, str>>, err: Error) -> TarError {
        TarError {
            desc: desc.into(),
            io: err,
        }
    }
}

impl error::Error for TarError {}

impl fmt::Display for TarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.desc.fmt(f)
    }
}

impl From<TarError> for Error {
    fn from(t: TarError) -> Error {
        Error::new(t.io.kind(), t)
    }
}
