use crate::stage1::{get_scripts, Stage1Tasks, PKG_SCRIPTS_DIR};

use common::pkg::{PkgDataFromDb, ScriptPhase};
use db::{
    enable_foreign_keys, pkg::DbOpsForInstalledPkg, transaction_op, Transaction, CORE_DB_PATH,
};
use ehandle::{lpm::LpmError, pkg::PackageErrorKind, ErrorCommons, MainError};
use logger::{info, success, warning};
use min_sqlite3_sys::prelude::*;
use std::{fs, path::Path};

trait PkgDeleteTasks {
    fn start_delete_task(&self) -> Result<(), LpmError<MainError>>;
}

impl PkgDeleteTasks for PkgDataFromDb {
    fn start_delete_task(&self) -> Result<(), LpmError<MainError>> {
        let db = Database::open(Path::new(CORE_DB_PATH))?;

        // Enable constraits to remove records that are related with package
        enable_foreign_keys(&db)?;

        transaction_op(&db, Transaction::Begin)?;

        let pkg_lib_dir = Path::new(PKG_SCRIPTS_DIR).join(&self.meta_fields.meta.name);
        let scripts = get_scripts(&pkg_lib_dir.join("scripts"))?;

        if let Err(err) = scripts.execute_script(ScriptPhase::PreDelete) {
            transaction_op(&db, Transaction::Rollback)?;
            return Err(err);
        }

        info!("Syncing with package database..");
        if self.delete_from_db(&db).is_err() {
            transaction_op(&db, Transaction::Rollback)?;

            return Err(
                PackageErrorKind::DeletionFailed(self.meta_fields.meta.name.clone())
                    .to_lpm_err()
                    .into(),
            );
        };

        info!("Deleting package files from system..");
        for file in &self.meta_fields.files.0 {
            if Path::new(&file.path).exists() {
                fs::remove_file(&file.path)?;
            } else {
                warning!("Path -> {} <- is not exists", file.path);
            }
        }

        if Path::new(&pkg_lib_dir).exists() {
            fs::remove_dir_all(pkg_lib_dir)?;
        }

        if let Err(err) = scripts.execute_script(ScriptPhase::PostDelete) {
            transaction_op(&db, Transaction::Rollback)?;
            return Err(err);
        }

        transaction_op(&db, Transaction::Commit)?;
        db.close();
        info!("Deletion transaction completed.");

        Ok(())
    }
}

pub fn delete_lod(pkg_name: &str) -> Result<(), LpmError<MainError>> {
    let db = Database::open(Path::new(CORE_DB_PATH))?;
    let pkg = PkgDataFromDb::load(&db, pkg_name)?;
    db.close();

    info!("Package deletion started for {}", pkg_name);
    pkg.start_delete_task()?;
    success!("Operation successfully completed.");
    Ok(())
}
