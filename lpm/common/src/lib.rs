pub mod lpm_version;
pub mod meta;
pub mod pkg;
pub mod system;
pub mod version;

// re-exports
pub use meta::Files;

pub trait ParserTasks {
    fn deserialize(path: &str) -> Self;
}

// For non-binary packages
pub const NO_ARCH: &str = "no-arch";

// Supported CPU architectures
#[cfg(target_arch = "x86_64")]
pub const SYSTEM_ARCH: &str = "amd64";
#[cfg(target_arch = "arm")]
pub const SYSTEM_ARCH: &str = "arm";

#[macro_export]
macro_rules! de_required_field {
    ($json: expr, $field: expr) => {
        match $json {
            Some(val) => val,
            None => {
                return Err(format!(
                    "Field '{}' is required and must be provided.",
                    $field
                ))
            }
        }
    };
}

#[macro_export]
macro_rules! some_or_error {
    ($fn: expr, $log: expr, $($args: tt)+) => {
        match $fn {
            Some(val) => val,
            None => panic!("{}", format!($log, $($args)+)),
        }
    };
    ($fn: expr, $log: expr) => {
        match $fn {
            Some(val) => val,
            None => panic!("{}", format!($log)),
        }

    }
}
