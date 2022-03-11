#![forbid(unsafe_code)]

use ehandle::RuntimeError;

const EXTRACTION_OUTPUT_PATH: &str = "/var/cache/lpm";

pub mod extraction;
pub mod installation;
pub mod pkg;
pub mod validation;

pub trait ExtractionTasks {
    fn start_extraction(&mut self) -> Result<(), RuntimeError>;
    fn get_pkg_output_path(&self) -> String;
    fn half_extract(&self) -> Result<(), RuntimeError>;
    fn extract_meta_and_program(&self) -> Result<(), RuntimeError>;
    fn read_pkg_data(&mut self);
    fn cleanup(&self) -> Result<(), RuntimeError>;
}

pub trait InstallationTasks {
    fn start_installation(&mut self) -> Result<(), RuntimeError>;
    fn install_program(&self) -> Result<(), RuntimeError>;
}

pub trait ValidationTasks {
    fn start_validations(&self) -> Result<(), RuntimeError>;
}
