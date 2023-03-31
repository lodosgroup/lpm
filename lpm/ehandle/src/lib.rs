use lpm::LpmError;

#[macro_export]
macro_rules! simple_e_fmt {
    ($format: expr, $($args: tt)+) => { format!($format, $($args)+) };
    ($format: expr) => { format!($format) }
}

pub trait ErrorCommons<T> {
    fn as_str(&self) -> &str;
    fn to_err(&self) -> T;
    #[track_caller]
    fn to_lpm_err(&self) -> LpmError<T>;
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct MainError {
    kind: String,
    reason: String,
}

pub mod db;
mod io;
pub mod lpm;
pub mod module;
pub mod pkg;
