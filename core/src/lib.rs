#![forbid(unsafe_code)]

use parser::version::VersionStruct;

const EXTRACTION_OUTPUT_PATH: &str = "/var/cache/lpm";

const _V_MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
const _V_MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
const _V_PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");

#[inline]
pub fn get_lpm_version() -> VersionStruct {
    VersionStruct {
        major: _V_MAJOR.parse().unwrap(),
        minor: _V_MINOR.parse().unwrap(),
        patch: _V_PATCH.parse().unwrap(),
        tag: None,
        readable_format: format!("{}.{}.{}", _V_MAJOR, _V_MINOR, _V_PATCH),
    }
}

pub mod extraction;
pub mod installation;
pub mod pkg;
pub mod validation;
