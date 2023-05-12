use common::pkg::PkgDataFromFs;
use db::{pkg::DbOpsForBuildFile, transaction_op, Transaction, DB_PATH};
use ehandle::{lpm::LpmError, MainError};
use logger::{debug, info, success};
use min_sqlite3_sys::prelude::*;
use std::{
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
};

use crate::{
    extract::{get_pkg_tmp_output_path, PkgExtractTasks},
    validate::PkgValidateTasks,
};

trait PkgInstallTasks {
    fn start_install_task(path: &str) -> Result<(), LpmError<MainError>>;
    fn copy_programs(&self) -> Result<(), LpmError<MainError>>;
    fn install_program(&self) -> Result<(), LpmError<MainError>>;
}

impl PkgInstallTasks for PkgDataFromFs {
    fn start_install_task(path: &str) -> Result<(), LpmError<MainError>> {
        let pkg_path = PathBuf::from(path);

        info!("Extracting..");
        let pkg = PkgDataFromFs::start_extract_task(&pkg_path)?;
        info!("Validating files..");
        pkg.start_validate_task()?;

        let db = Database::open(Path::new(DB_PATH))?;
        info!("Syncing with package database..");
        pkg.insert_to_db(&db)?;

        info!("Installing package files into system..");
        match pkg.install_program() {
            Ok(_) => {}
            Err(err) => {
                transaction_op(&db, Transaction::Rollback)?;
                return Err(err);
            }
        };

        info!("Cleaning temporary files..");
        match pkg.cleanup() {
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
        info!("Installation transaction completed.");

        Ok(())
    }

    #[inline(always)]
    fn install_program(&self) -> Result<(), LpmError<MainError>> {
        self.copy_programs()
    }

    fn copy_programs(&self) -> Result<(), LpmError<MainError>> {
        let source_path = get_pkg_tmp_output_path(&self.path).join("program");

        for file in &self.meta_dir.files.0 {
            let destination = Path::new("/").join(&file.path);
            create_dir_all(destination.parent().unwrap())?;

            let from = source_path.join(&file.path);

            debug!("Copying {} -> {}", from.display(), destination.display());

            fs::copy(from, destination)?;
        }

        Ok(())
    }
}

pub fn install_lod(pkg_path: &str) -> Result<(), LpmError<MainError>> {
    info!("Package installation started for {}", pkg_path);
    PkgDataFromFs::start_install_task(pkg_path)?;
    success!("Operation successfully completed.");
    Ok(())
}
