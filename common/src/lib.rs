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
macro_rules! try_or_error {
    ($fn: expr) => {
        match $fn {
            Result::Ok(val) => val,
            Result::Err(err) => {
                term::error!("{:?}", err);
                // Terminate app with panic code
                std::process::exit(101);
            }
        }
    };
}

#[macro_export]
macro_rules! log_and_panic {
    ($log: expr) => {
        term::error!("{}", format!($log));

        // Terminate app with panic code
        std::process::exit(101);
    };
    ($log: expr, $($args: tt)+) => {
        term::error!("{}", format!($log, $($args)+));

        // Terminate app with panic code
        std::process::exit(101);
    };
}
