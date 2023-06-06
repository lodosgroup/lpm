use crate::{
    extract::{get_pkg_tmp_output_path, PkgExtractTasks},
    repository::find_most_recent_pkg_index,
    stage1::{Stage1Tasks, PKG_SCRIPTS_DIR},
    validate::PkgValidateTasks,
};

use common::{
    download_file,
    pkg::{PkgDataFromFs, ScriptPhase},
};
use db::{
    pkg::{is_package_exists, DbOpsForBuildFile},
    transaction_op, Transaction, CORE_DB_PATH,
};
use ehandle::{lpm::LpmError, pkg::PackageErrorKind, ErrorCommons, MainError};
use logger::{debug, info, success};
use min_sqlite3_sys::prelude::*;
use std::{
    fs::{self, create_dir_all, remove_file},
    path::{Path, PathBuf},
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

        let db = Database::open(Path::new(CORE_DB_PATH))?;
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
        let pkg_scripts_path = Path::new(PKG_SCRIPTS_DIR)
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

pub fn install_from_repository(pkg_name: &str) -> Result<(), LpmError<MainError>> {
    let db = Database::open(Path::new(CORE_DB_PATH))?;
    if is_package_exists(&db, pkg_name)? {
        return Err(PackageErrorKind::AlreadyInstalled(pkg_name.to_owned()).to_lpm_err())?;
    }
    db.close();

    let index = find_most_recent_pkg_index(pkg_name)?;
    let pkg_path = index.pkg_output_path(super::EXTRACTION_OUTPUT_PATH);

    download_file(&index.pkg_url(), &pkg_path)?;

    let pkg_path_as_string = pkg_path.display().to_string();
    install_from_lod_file(&pkg_path_as_string)?;
    remove_file(pkg_path)?;

    Ok(())
}

pub fn install_from_lod_file(pkg_path: &str) -> Result<(), LpmError<MainError>> {
    info!("Package installation started for {}", pkg_path);
    PkgDataFromFs::start_install_task(pkg_path)?;
    success!("Operation successfully completed.");
    Ok(())
}
