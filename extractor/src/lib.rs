#![forbid(unsafe_code)]

const EXTRACTION_OUTPUT_PATH: &str = "/var/cache/lpm/";

pub mod cleanup;
pub mod pkg;

pub trait ExtractionTasks {
    fn half_extract(&self) -> Result<(), std::io::Error>;
}
