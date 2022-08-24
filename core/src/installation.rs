use crate::{extraction::ExtractionTasks, validation::ValidationTasks};
use common::pkg::LodPkg;
use db::{pkg::LodPkgCoreDbOps, transaction_op, Transaction, DB_PATH};
use ehandle::{lpm::LpmError, MainError};
use min_sqlite3_sys::prelude::*;
use std::{
    fs::{self, create_dir_all},
    io,
    path::Path,
};

pub trait InstallationTasks {
    fn copy_programs(&self) -> Result<(), LpmError<io::Error>>;
    fn start_installation(&mut self) -> Result<(), LpmError<MainError>>;
    fn install_program(&self) -> Result<(), LpmError<io::Error>>;
}

impl<'a> InstallationTasks for LodPkg<'a> {
    fn start_installation(&mut self) -> Result<(), LpmError<MainError>> {
        self.start_extraction()?;
        self.start_validations()?;

        let db = Database::open(Path::new(DB_PATH))?;
        self.insert_to_db(&db)?;

        match self.install_program() {
            Ok(_) => {}
            Err(err) => {
                transaction_op(&db, Transaction::Rollback)?;
                return Err(LpmError::from(err));
            }
        };

        match self.cleanup() {
            Ok(_) => {}
            Err(err) => {
                transaction_op(&db, Transaction::Rollback)?;
                return Err(err.into());
            }
        };

        match transaction_op(&db, Transaction::Commit) {
            Ok(_) => {}
            Err(err) => {
                transaction_op(&db, Transaction::Rollback)?;
                return Err(err.into());
            }
        };

        db.close();

        Ok(())
    }

    #[inline(always)]
    fn install_program(&self) -> Result<(), LpmError<io::Error>> {
        self.copy_programs()
    }

    fn copy_programs(&self) -> Result<(), LpmError<io::Error>> {
        let source_path = super::EXTRACTION_OUTPUT_PATH.to_string()
            + "/"
            + self.path.unwrap().file_stem().unwrap().to_str().unwrap()
            + "/program/";

        for file in &self.meta_dir.as_ref().unwrap().files.0 {
            let destination_path = Path::new("/").join(&file.path);
            create_dir_all(destination_path.parent().unwrap()).unwrap();

            fs::copy(source_path.clone() + &file.path, destination_path)?;
        }

        Ok(())
    }
}
