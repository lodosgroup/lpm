#![forbid(unsafe_code)]

const EXTRACTION_OUTPUT_PATH: &str = "/var/cache/lpm";

const _V_MAJOR: i8 = 0;
const _V_MINOR: i8 = 0;
const _V_PATCH: i8 = 0;

pub mod extraction;
pub mod installation;
pub mod pkg;
pub mod validation;
