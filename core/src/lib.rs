#![forbid(unsafe_code)]

use std::{error, io};

const EXTRACTION_OUTPUT_PATH: &str = "/var/cache/lpm";

const _V_MAJOR: &i8 = 0;
const _V_MINOR: &i8 = 0;
const _V_PATCH: &i8 = 0;

pub mod extraction;
pub mod installation;
pub mod pkg;
pub mod validation;

pub trait ExtractionTasks {
    fn start_extraction(&mut self) -> Result<(), io::Error>;
    fn get_pkg_output_path(&self) -> String;
    fn half_extract(&self) -> Result<(), io::Error>;
    fn extract_meta_and_program(&self) -> Result<(), io::Error>;
    fn read_pkg_data(&mut self);
    fn cleanup(&self) -> Result<(), io::Error>;
}

pub trait InstallationTasks {
    fn start_installation(&mut self) -> Result<(), Box<dyn error::Error>>;
    fn install_program(&self) -> Result<(), io::Error>;
}

pub trait ValidationTasks {
    fn start_validations(&self) -> Result<(), Box<dyn error::Error>>;
}
