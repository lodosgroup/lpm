use common::pkg::LodPkg;
use db::{pkg::LodPkgCoreDbOps, transaction_op, Transaction, DB_PATH};
use ehandle::{lpm::LpmError, pkg::PackageErrorKind, ErrorCommons, MainError};
use min_sqlite3_sys::prelude::*;
use std::{fs, path::Path};
use term::{info, warning};

pub trait DeletionTasks {
    fn start_deletion(&self) -> Result<(), LpmError<MainError>>;
}

impl<'a> DeletionTasks for LodPkg<'a> {
    fn start_deletion(&self) -> Result<(), LpmError<MainError>> {
        let meta_dir = self
            .meta_dir
            .as_ref()
            .ok_or_else(|| PackageErrorKind::MetaDirCouldNotLoad.to_lpm_err())?;

        let db = Database::open(Path::new(DB_PATH))?;
        transaction_op(&db, Transaction::Begin)?;

        info!("Syncing with package database..");
        match self.delete_from_db(&db) {
            Ok(_) => {}
            Err(_) => {
                transaction_op(&db, Transaction::Rollback)?;

                return Err(PackageErrorKind::DeletionFailed(meta_dir.meta.name.clone())
                    .to_lpm_err()
                    .into());
            }
        };

        info!("Deleting package files from system..");
        for file in &meta_dir.files.0 {
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
