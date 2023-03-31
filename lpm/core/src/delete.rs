use common::pkg::PkgDataFromDb;
use db::{enable_foreign_keys, pkg::DbOpsForInstalledPkg, transaction_op, Transaction, DB_PATH};
use ehandle::{lpm::LpmError, pkg::PackageErrorKind, ErrorCommons, MainError};
use logger::{info, success, warning};
use min_sqlite3_sys::prelude::*;
use std::{fs, path::Path};

trait PkgDeleteTasks {
    fn start_delete_task(&self) -> Result<(), LpmError<MainError>>;
}

impl PkgDeleteTasks for PkgDataFromDb {
    fn start_delete_task(&self) -> Result<(), LpmError<MainError>> {
        let db = Database::open(Path::new(DB_PATH))?;

        // Enable constraits to remove records that are related with package
        enable_foreign_keys(&db)?;

        transaction_op(&db, Transaction::Begin)?;

        info!("Syncing with package database..");
        match self.delete_from_db(&db) {
            Ok(_) => {}
            Err(_) => {
                transaction_op(&db, Transaction::Rollback)?;

                return Err(
                    PackageErrorKind::DeletionFailed(self.meta_dir.meta.name.clone())
                        .to_lpm_err()
                        .into(),
                );
            }
        };

        info!("Deleting package files from system..");
        for file in &self.meta_dir.files.0 {
            if Path::new(&file.path).exists() {
                fs::remove_file(file.path.clone())?;
            } else {
                warning!("Path -> {} <- is not exists", file.path);
            }
        }

        transaction_op(&db, Transaction::Commit)?;
        db.close();
        info!("Deletion transaction completed.");

        Ok(())
    }
}

pub fn delete_lod(pkg_name: &str) -> Result<(), LpmError<MainError>> {
    let db = Database::open(Path::new(DB_PATH))?;
    let pkg = PkgDataFromDb::load(&db, pkg_name)?;
    db.close();

    info!("Package deletion started for {}", pkg_name);
    pkg.start_delete_task()?;
    success!("Operation successfully completed.");
    Ok(())
}
