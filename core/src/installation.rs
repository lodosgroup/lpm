use std::{
    error,
    fs::{self, create_dir_all},
    io,
    path::Path,
};

use crate::{extraction::ExtractionTasks, pkg::LodPkg, validation::ValidationTasks};

pub trait InstallationTasks {
    fn copy_programs(&self) -> Result<(), io::Error>;
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
        self.copy_programs()
    }

    #[inline(always)]
    fn copy_programs(&self) -> Result<(), io::Error> {
        let source_path = super::EXTRACTION_OUTPUT_PATH.to_string()
            + "/"
            + self.path.file_stem().unwrap().to_str().unwrap()
            + "/program/";

        for file in &self.meta_dir.as_ref().unwrap().files.0 {
            let destination_path = Path::new("/").join(&file.path);
            create_dir_all(destination_path.parent().unwrap()).unwrap();

            fs::copy(source_path.clone() + &file.path, destination_path)?;
        }

        Ok(())
    }
}
