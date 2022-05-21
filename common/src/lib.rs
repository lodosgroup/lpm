pub mod lpm_version;
pub mod pkg;
pub mod meta;
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
