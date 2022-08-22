#![feature(io_error_more, io_error_uncategorized)]

#[macro_export]
macro_rules! simple_e_fmt {
    ($format: expr, $($args: tt)+) => { format!($format, $($args)+) };
    ($format: expr) => { format!($format) }
}

#[macro_export]
macro_rules! backtrace_e_fmt {
    ($format: expr, $($args: tt)+) => { ehandle::backtrace!($format, $($args)+) };
    ($format: expr) => { ehandle::backtrace!($format) }
}

#[macro_export]
macro_rules! backtrace {
    ($format: expr, $($args: tt)+) => {format! (concat! ("{}:{} >> ", $format), file!(), line!(), $($args)+)};
    ($format: expr) => {format! (concat! ("{}:{} >> ", $format), file!(), line!())}
}

pub trait ErrorCommons<T> {
    fn as_str(&self) -> &str;
    fn throw(&self) -> T;
}

#[non_exhaustive]
#[derive(Debug)]
pub enum RuntimeErrorKind {
    UnsupportedPlatform(Option<String>),
}

impl ErrorCommons<RuntimeError> for RuntimeErrorKind {
    #[inline(always)]
    fn as_str(&self) -> &str {
        match self {
            RuntimeErrorKind::UnsupportedPlatform(_) => "UnsupportedPlatform",
        }
    }

    #[inline(always)]
    fn throw(&self) -> RuntimeError {
        match self {
            Self::UnsupportedPlatform(ref err) => RuntimeError {
                kind: self.as_str().to_string(),
                reason: err
                    .as_ref()
                    .unwrap_or(&String::from(
                        "LodPM can only work on Linux based platforms.",
                    ))
                    .to_owned(),
            },
        }
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub kind: String,
    pub reason: String,
}

pub mod db;
mod io;
pub mod pkg;
pub mod lpm;
