#[macro_export]
macro_rules! simple_e_fmt {
    ($format: expr, $($args: tt)+) => { format!($format, $($args)+) };
    ($format: expr) => { format!($format) }
}

pub trait ErrorCommons<T> {
    fn as_str(&self) -> &str;
    fn throw(&self) -> T;
}

#[non_exhaustive]
#[derive(Debug)]
pub enum BuildtimeErrorKind {
    UnsupportedPlatform(Option<String>),
}

impl ErrorCommons<MainError> for BuildtimeErrorKind {
    #[inline]
    fn as_str(&self) -> &str {
        match self {
            BuildtimeErrorKind::UnsupportedPlatform(_) => "UnsupportedPlatform",
        }
    }

    fn throw(&self) -> MainError {
        match self {
            Self::UnsupportedPlatform(ref err) => MainError {
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

#[allow(dead_code)]
#[derive(Debug)]
pub struct MainError {
    kind: String,
    reason: String,
}

pub mod db;
mod io;
pub mod lpm;
pub mod pkg;
