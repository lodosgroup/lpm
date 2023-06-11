use crate::stage1::{get_scripts, Stage1Tasks, PKG_SCRIPTS_DIR};

use common::pkg::{PkgDataFromDb, ScriptPhase};
use db::{enable_foreign_keys, pkg::DbOpsForInstalledPkg, transaction_op, Transaction};
use ehandle::{lpm::LpmError, pkg::PackageErrorKind, ErrorCommons, MainError};
use logger::{info, success, warning};
use min_sqlite3_sys::prelude::Database;
use std::{fs, path::Path};

trait PkgDeleteTasks {
    fn start_delete_task(&self, core_db: &Database) -> Result<(), LpmError<MainError>>;
}

impl PkgDeleteTasks for PkgDataFromDb {
    fn start_delete_task(&self, core_db: &Database) -> Result<(), LpmError<MainError>> {
        // Enable constraits to remove records that are related with package
        enable_foreign_keys(core_db)?;

        transaction_op(core_db, Transaction::Begin)?;

        let pkg_lib_dir = Path::new(PKG_SCRIPTS_DIR).join(&self.meta_fields.meta.name);
        let scripts = get_scripts(&pkg_lib_dir.join("scripts"))?;

        if let Err(err) = scripts.execute_script(ScriptPhase::PreDelete) {
            transaction_op(core_db, Transaction::Rollback)?;
            return Err(err);
        }

        info!("Syncing with package database..");
        if self.delete_from_db(core_db).is_err() {
            transaction_op(core_db, Transaction::Rollback)?;

            return Err(
                PackageErrorKind::DeletionFailed(self.meta_fields.meta.name.clone()).to_lpm_err(),
            )?;
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
            transaction_op(core_db, Transaction::Rollback)?;
            return Err(err);
        }

        transaction_op(core_db, Transaction::Commit)?;
        info!("Deletion transaction completed.");

        Ok(())
    }
}

pub fn delete_lod(core_db: &Database, pkg_name: &str) -> Result<(), LpmError<MainError>> {
    let pkg = PkgDataFromDb::load(core_db, pkg_name)?;

    if let Some(src_id) = pkg.src_pkg_package_id {
        let main_pkg_name = PkgDataFromDb::get_name_by_id(core_db, src_id)?;
        if let Some(main_pkg_name) = main_pkg_name {
            return Err(PackageErrorKind::DependencyOfAnotherPackage {
                package: pkg_name.to_owned(),
                depends_on: main_pkg_name,
            }
            .to_lpm_err())?;
        }
    };

    info!("Package deletion started for {}", pkg_name);
    pkg.start_delete_task(core_db)?;
    success!("Operation successfully completed.");
    Ok(())
}
