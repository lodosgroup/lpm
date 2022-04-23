#![forbid(unsafe_code)]

pub mod lpm_version;
pub mod pkg;

// re-exports
pub use parser::meta::Files;

// For non-binary packages
pub const NO_ARCH: &str = "no-arch";

// Supported CPU architectures
#[cfg(target_arch = "x86_64")]
pub const SYSTEM_ARCH: &str = "amd64";
#[cfg(target_arch = "arm")]
pub const SYSTEM_ARCH: &str = "arm";
