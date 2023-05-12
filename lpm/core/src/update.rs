use crate::{extract::get_pkg_tmp_output_dir, validate::PkgValidateTasks, PkgExtractTasks};

use common::{
    pkg::{PkgDataFromDb, PkgDataFromFs},
    Files,
};
use db::{
    pkg::{DbOpsForBuildFile, DbOpsForInstalledPkg},
    transaction_op, Transaction, DB_PATH,
};
use ehandle::{lpm::LpmError, MainError};
use logger::{debug, info, success, warning};
use min_sqlite3_sys::prelude::{Connection, Database};
use std::{
    fs::{self, create_dir_all},
    path::Path,
};

trait PkgUpdateTasks {
    fn start_update_task(&mut self, to: &mut PkgDataFromFs) -> Result<(), LpmError<MainError>>;

    fn compare_and_update_files_on_fs(
        &mut self,
        pkg_path: String,
        new_files: Files,
    ) -> Result<(), LpmError<MainError>>;
}

impl PkgUpdateTasks for PkgDataFromDb {
    fn start_update_task(&mut self, to_pkg: &mut PkgDataFromFs) -> Result<(), LpmError<MainError>> {
        debug!("Comparing versions..");
        let operation = match self
            .meta_dir
            .meta
            .version
            .compare(&to_pkg.meta_dir.meta.version)
        {
            std::cmp::Ordering::Less => {
                // TODO Ask for upgrading
                "Package upgrade"
            }
            std::cmp::Ordering::Greater => {
                // TODO Ask for downgrading
                "Package downgrade"
            }
            std::cmp::Ordering::Equal => {
                warning!(
                    "Requested package has exactly same version with the one currently installed."
                );

                return Ok(());
            }
        };

        to_pkg.start_validate_task()?;
        let source_path = get_pkg_tmp_output_dir(&to_pkg.path) + "/program/";

        info!("Applying package differences to the system..");
        self.compare_and_update_files_on_fs(source_path, to_pkg.meta_dir.files.clone())?;

        let db = Database::open(Path::new(DB_PATH))?;
        info!("Syncing with package database..");
        to_pkg.update_existing_pkg(&db, self.pkg_id)?;

        info!("Cleaning temporary files..");
        match to_pkg.cleanup() {
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
        info!("{} transaction completed.", operation);

        db.close();

        Ok(())
    }

    /// Loops over target files, copies each one of them unless they are
    /// already exists in the system, ignores otherwise.
    fn compare_and_update_files_on_fs(
        &mut self,
        pkg_path: String,
        new_files: Files,
    ) -> Result<(), LpmError<MainError>> {
        for file in new_files.0.iter() {
            let file_index = self
                .meta_dir
                .files
                .0
                .iter()
                .position(|f| f.path == "/".to_owned() + &file.path);
            if let Some(file_index) = file_index {
                let found_file = &self.meta_dir.files.0[file_index];

                // if both files are exactly the same
                if found_file.checksum_algorithm == file.checksum_algorithm
                    && found_file.checksum == file.checksum
                {
                    debug!(
                        "File /{} has same checksum in target package, ignoring it.",
                        file.path
                    );
                    self.meta_dir.files.0.remove(file_index);
                    continue;
                } else {
                    debug!(
                        "Updating /{} with the other version of it in the target package.",
                        file.path
                    );
                    fs::remove_file(&found_file.path)?;
                    self.meta_dir.files.0.remove(file_index);

                    let destination_path = Path::new("/").join(&file.path);
                    fs::copy(pkg_path.clone() + &file.path, destination_path)?;
                }
            }
            // File is not included in the old pkg version
            else {
                debug!("Adding /{} to the system.", file.path);
                let destination_path = Path::new("/").join(&file.path);
                // Ensure the target dir path
                create_dir_all(destination_path.parent().unwrap())?;
                fs::copy(pkg_path.clone() + &file.path, destination_path)?;
            }
        }

        for file in self.meta_dir.files.0.iter() {
            debug!(
                "Removing {} since it's not needed in target package",
                file.path
            );
            fs::remove_file(&file.path)?;
        }

        Ok(())
    }
}

pub fn update_lod(pkg_name: &str, pkg_path: &str) -> Result<(), LpmError<MainError>> {
    let db = Database::open(Path::new(DB_PATH))?;
    let mut old_pkg = PkgDataFromDb::load(&db, pkg_name)?;
    db.close();

    let mut requested_pkg = PkgDataFromFs::start_extract_task(Path::new(pkg_path))?;

    info!("Package update started for {}", pkg_name);
    old_pkg.start_update_task(&mut requested_pkg)?;
    success!("Operation successfully completed.");

    Ok(())
}
