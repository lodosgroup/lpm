use common::pkg::{PkgDataFromFs, ScriptPhase};
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
    stage1::Stage1Tasks,
    validate::PkgValidateTasks,
};

trait PkgInstallTasks {
    fn start_install_task(path: &str) -> Result<(), LpmError<MainError>>;
    fn copy_programs(&self) -> Result<(), LpmError<MainError>>;
    fn copy_scripts(&self) -> Result<(), LpmError<MainError>>;
    fn install(&self) -> Result<(), LpmError<MainError>>;
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

        if let Err(err) = pkg.scripts.execute_script(ScriptPhase::PreInstall) {
            transaction_op(&db, Transaction::Rollback)?;
            return Err(err);
        }

        info!("Installing package files into system..");
        if let Err(err) = pkg.install() {
            transaction_op(&db, Transaction::Rollback)?;
            return Err(err);
        };

        info!("Cleaning temporary files..");
        if let Err(err) = pkg.cleanup() {
            transaction_op(&db, Transaction::Rollback)?;
            return Err(err.into());
        };

        if let Err(err) = transaction_op(&db, Transaction::Commit) {
            transaction_op(&db, Transaction::Rollback)?;
            return Err(err.into());
        };

        if let Err(err) = pkg.scripts.execute_script(ScriptPhase::PostInstall) {
            transaction_op(&db, Transaction::Rollback)?;
            return Err(err);
        }

        db.close();
        info!("Installation transaction completed.");

        Ok(())
    }

    #[inline(always)]
    fn install(&self) -> Result<(), LpmError<MainError>> {
        self.copy_scripts()?;
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

    fn copy_scripts(&self) -> Result<(), LpmError<MainError>> {
        let pkg_scripts_path = Path::new("/var/lib/lpm/pkg/")
            .join(&self.meta_dir.meta.name)
            .join("scripts");

        std::fs::create_dir_all(&pkg_scripts_path)?;

        for script in &self.scripts {
            let destination = &pkg_scripts_path.join(script.path.file_name().unwrap());

            debug!(
                "Copying {} -> {}",
                script.path.display(),
                destination.display()
            );

            fs::copy(&script.path, destination)?;
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
