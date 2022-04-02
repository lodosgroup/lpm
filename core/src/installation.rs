use std::{
    error,
    fs::{self, create_dir_all},
    io,
    path::Path,
};

use crate::{extraction::ExtractionTasks, pkg::LodPkg, validation::ValidationTasks};

pub trait InstallationTasks {
    fn start_installation(&mut self) -> Result<(), Box<dyn error::Error>>;
    fn install_program(&self) -> Result<(), io::Error>;
}

impl<'a> InstallationTasks for LodPkg<'a> {
    fn start_installation(&mut self) -> Result<(), Box<dyn error::Error>> {
        self.start_extraction()?;
        self.start_validations()?;
        self.install_program()?;
        self.cleanup()?;

        Ok(())
    }

    fn install_program(&self) -> Result<(), io::Error> {
        let src = super::EXTRACTION_OUTPUT_PATH.to_string()
            + "/"
            + self.path.file_stem().unwrap().to_str().unwrap()
            + "/program/";

        copy_recursively(&src, "/")?;

        Ok(())
    }
}

#[inline(always)]
fn copy_recursively(src: &str, destination: &str) -> Result<(), io::Error> {
    create_dir_all(destination.clone())?;

    let src = Path::new(src);
    let destination = Path::new(destination);

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_recursively(
                entry.path().to_str().unwrap(),
                destination.join(entry.file_name()).to_str().unwrap(),
            )?;
        } else {
            fs::copy(
                entry.path(),
                destination.join(entry.file_name()).to_str().unwrap(),
            )?;
        }
    }

    Ok(())
}
