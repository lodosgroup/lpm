#![forbid(unsafe_code)]

const EXTRACTION_OUTPUT_PATH: &str = "/var/cache/lpm";

const _V_MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
const _V_MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
const _V_PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");

pub mod extraction;
pub mod installation;
pub mod pkg;
pub mod validation;
