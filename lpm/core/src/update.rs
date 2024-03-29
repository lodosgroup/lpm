use crate::{
    extract::get_pkg_tmp_output_path,
    repository::find_pkg_index,
    stage1::{get_scripts, Stage1Tasks, PKG_SCRIPTS_DIR},
    validate::PkgValidateTasks,
    Ctx, PkgExtractTasks,
};

use common::{
    ctx_confirmation_check, download_file,
    pkg::{PkgDataFromDb, PkgDataFromFs, PkgToQuery, ScriptPhase},
    Files,
};
use db::{
    enable_core_db_wal1,
    pkg::{DbOpsForBuildFile, DbOpsForInstalledPkg},
    transaction_op, Transaction,
};
use ehandle::{lpm::LpmError, repository::RepositoryErrorKind, ErrorCommons, MainError};
use logger::{debug, info, warning};
use min_sqlite3_sys::prelude::Database;
use std::{
    fs::{self, create_dir_all, remove_file},
    path::Path,
    sync::Arc,
    thread,
};

trait PkgUpdateTasks {
    fn start_update_task(
        &mut self,
        core_db: &Database,
        to: &mut PkgDataFromFs,
    ) -> Result<(), LpmError<MainError>>;

    fn compare_and_update_files_on_fs(
        &mut self,
        pkg_path: &Path,
        new_files: Files,
    ) -> Result<(), LpmError<MainError>>;
}

impl PkgUpdateTasks for PkgDataFromDb {
    fn start_update_task(
        &mut self,
        core_db: &Database,
        to_pkg: &mut PkgDataFromFs,
    ) -> Result<(), LpmError<MainError>> {
        debug!("Comparing versions..");

        let (pre_script, post_script) = match self
            .meta_fields
            .meta
            .version
            .compare(&to_pkg.meta_dir.meta.version)
        {
            std::cmp::Ordering::Less => {
                // TODO Ask for upgrading
                (ScriptPhase::PreUpgrade, ScriptPhase::PostUpgrade)
            }
            std::cmp::Ordering::Greater => {
                // TODO Ask for downgrading
                (ScriptPhase::PreDowngrade, ScriptPhase::PostDowngrade)
            }
            std::cmp::Ordering::Equal => {
                warning!(
                    "Requested package has exactly same version with the one currently installed."
                );

                return Ok(());
            }
        };

        let pkg_lib_dir = Path::new(PKG_SCRIPTS_DIR).join(&self.meta_fields.meta.name);
        let scripts = get_scripts(&pkg_lib_dir.join("scripts"))?;

        to_pkg.start_validate_task()?;
        let source_path = get_pkg_tmp_output_path(&to_pkg.path).join("program");

        if let Err(err) = scripts.execute_script(vec![], pre_script) {
            transaction_op(core_db, Transaction::Rollback)?;
            return Err(err);
        }

        info!("Applying package differences to the system..");
        self.compare_and_update_files_on_fs(&source_path, to_pkg.meta_dir.files.clone())?;

        info!("Syncing with package database..");
        to_pkg.update_existing_pkg(core_db, self.pkg_id, to_pkg.meta_dir.meta.get_group_id())?;

        if let Err(err) = scripts.execute_script(vec![], post_script) {
            transaction_op(core_db, Transaction::Rollback)?;
            return Err(err);
        }

        if let Err(err) = transaction_op(core_db, Transaction::Commit) {
            transaction_op(core_db, Transaction::Rollback)?;
            return Err(err)?;
        };
        info!("Update transaction completed.");

        Ok(())
    }

    /// Loops over target files, copies each one of them unless they are
    /// already exists in the system, ignores otherwise.
    fn compare_and_update_files_on_fs(
        &mut self,
        pkg_path: &Path,
        new_files: Files,
    ) -> Result<(), LpmError<MainError>> {
        for file in new_files.0.iter() {
            let file_index = self
                .meta_fields
                .files
                .0
                .iter()
                .position(|f| f.path == "/".to_owned() + &file.path);
            if let Some(file_index) = file_index {
                let found_file = &self.meta_fields.files.0[file_index];

                // if both files are exactly the same
                if found_file.checksum_algorithm == file.checksum_algorithm
                    && found_file.checksum == file.checksum
                {
                    debug!(
                        "File /{} has same checksum in target package, ignoring it.",
                        file.path
                    );
                    self.meta_fields.files.0.remove(file_index);
                    continue;
                } else {
                    debug!(
                        "Updating /{} with the other version of it in the target package.",
                        file.path
                    );
                    fs::remove_file(&found_file.path)?;
                    self.meta_fields.files.0.remove(file_index);

                    let destination_path = Path::new("/").join(&file.path);
                    fs::copy(pkg_path.join(&file.path), destination_path)?;
                }
            }
            // File is not included in the old pkg version
            else {
                debug!("Adding /{} to the system.", file.path);
                let destination_path = Path::new("/").join(&file.path);
                // Ensure the target dir path
                create_dir_all(destination_path.parent().unwrap())?;
                fs::copy(pkg_path.join(&file.path), destination_path)?;
            }
        }

        for file in self.meta_fields.files.0.iter() {
            debug!(
                "Removing {} since it's not needed in target package",
                file.path
            );
            fs::remove_file(&file.path)?;
        }

        Ok(())
    }
}

pub fn update_pkgs_from_repository(ctx: Ctx) -> Result<(), LpmError<MainError>> {
    enable_core_db_wal1(&ctx.core_db)?;

    let pkgs = PkgDataFromDb::load_all_main_packages(&ctx.core_db)?;
    let mut old_pkgs = vec![];

    for pkg in pkgs {
        let pkg_to_query = PkgToQuery {
            name: pkg.meta_fields.meta.name.clone(),
            condition: Default::default(),
            major: None,
            minor: None,
            patch: None,
            tag: None,
        };

        let index_db_list = db::get_repositories(&ctx.core_db)?;

        if index_db_list.is_empty() {
            info!("No repository has been found within the database.");
            return Err(RepositoryErrorKind::PackageNotFound(pkg_to_query.name).to_lpm_err())?;
        }

        let index = find_pkg_index(&index_db_list, &pkg_to_query)?;

        if pkg.meta_fields.meta.version.compare(&index.version) == std::cmp::Ordering::Less {
            old_pkgs.push(pkg);
        }
    }

    if old_pkgs.is_empty() {
        info!("All packages are already up to date.");
        return Ok(());
    }

    // TODO
    // add new versions that will be installed
    // package size is missing
    // total installation size is missing
    // use colors
    println!("\nPackage list to be updated:");
    for old_pkg in &old_pkgs {
        println!("  - {}", old_pkg.group_id);
    }
    println!();
    ctx_confirmation_check!(ctx);

    let core_db = Arc::new(&ctx.core_db);
    thread::scope(|s| -> Result<(), LpmError<MainError>> {
        for mut old_pkg in old_pkgs {
            let core_db = core_db.clone();

            let index_db_list = db::get_repositories(&ctx.core_db)?;

            s.spawn(move || -> Result<(), LpmError<MainError>> {
                let pkg_to_query = PkgToQuery {
                    name: old_pkg.meta_fields.meta.name.clone(),
                    condition: Default::default(),
                    major: None,
                    minor: None,
                    patch: None,
                    tag: None,
                };

                if index_db_list.is_empty() {
                    info!("No repository has been found within the database.");
                    return Err(
                        RepositoryErrorKind::PackageNotFound(pkg_to_query.name).to_lpm_err()
                    )?;
                }

                let index = find_pkg_index(&index_db_list, &pkg_to_query)?;
                let pkg_path = index.pkg_output_path(super::EXTRACTION_OUTPUT_PATH);

                download_file(&index.pkg_url(), &pkg_path)?;
                let mut requested_pkg = PkgDataFromFs::start_extract_task(&pkg_path)?;

                info!("Package update started for {}", pkg_to_query.name);
                old_pkg.start_update_task(&core_db, &mut requested_pkg)?;

                Ok(())
            });
        }

        Ok(())
    })
}

pub fn update_pkg_from_repository(ctx: Ctx, pkg_name: &str) -> Result<(), LpmError<MainError>> {
    enable_core_db_wal1(&ctx.core_db)?;

    // ensure the pkg exists
    let mut old_pkg = PkgDataFromDb::load(&ctx.core_db, pkg_name)?;

    let pkg_to_query = PkgToQuery {
        name: pkg_name.to_owned(),
        condition: Default::default(),
        major: None,
        minor: None,
        patch: None,
        tag: None,
    };

    let index_db_list = db::get_repositories(&ctx.core_db)?;

    if index_db_list.is_empty() {
        info!("No repository has been found within the database.");
        return Err(RepositoryErrorKind::PackageNotFound(pkg_to_query.name).to_lpm_err())?;
    }

    let index = find_pkg_index(&index_db_list, &pkg_to_query)?;

    if old_pkg.meta_fields.meta.version.compare(&index.version) == std::cmp::Ordering::Equal {
        info!("{} is up to date", pkg_name);
        return Ok(());
    }

    let pkg_path = index.pkg_output_path(super::EXTRACTION_OUTPUT_PATH);

    {
        // TODO
        // package size is missing
        // total installation size is missing
        // use colors
        println!("\nPackage list to be updated:");
        println!("  - {}", index.get_group_id());
        println!();
    }

    ctx_confirmation_check!(ctx);

    download_file(&index.pkg_url(), &pkg_path)?;

    let mut requested_pkg = PkgDataFromFs::start_extract_task(&pkg_path)?;

    info!("Package update started for {}", pkg_name);
    old_pkg.start_update_task(&ctx.core_db, &mut requested_pkg)?;

    remove_file(pkg_path)?;

    Ok(())
}

pub fn update_pkg_from_lod_file(
    ctx: Ctx,
    pkg_name: &str,
    pkg_path: &str,
) -> Result<(), LpmError<MainError>> {
    enable_core_db_wal1(&ctx.core_db)?;

    let mut old_pkg = PkgDataFromDb::load(&ctx.core_db, pkg_name)?;
    let mut requested_pkg = PkgDataFromFs::start_extract_task(Path::new(pkg_path))?;

    {
        // TODO
        // package size is missing
        // total installation size is missing
        // use colors
        println!("\nPackage list to be updated:");
        println!("  - {}", requested_pkg.meta_dir.meta.get_group_id());
        println!();
    }
    ctx_confirmation_check!(ctx);

    info!("Package update started for {}", pkg_name);
    old_pkg.start_update_task(&ctx.core_db, &mut requested_pkg)?;

    Ok(())
}
