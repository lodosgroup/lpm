use crate::{
    extract::{get_pkg_tmp_output_path, PkgExtractTasks},
    repository::find_pkg_index,
    stage1::{Stage1Tasks, PKG_SCRIPTS_DIR},
    validate::PkgValidateTasks,
};

use common::{
    download_file,
    pkg::{PkgDataFromFs, PkgToQuery, ScriptPhase},
    some_or_error,
};
use db::{
    pkg::{is_package_exists, DbOpsForBuildFile},
    transaction_op, PkgIndex, Transaction,
};
use ehandle::{
    lpm::LpmError, pkg::PackageErrorKind, repository::RepositoryErrorKind, ErrorCommons, MainError,
};
use logger::{debug, info, warning};
use min_sqlite3_sys::prelude::*;
use std::{
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
    thread,
};

trait PkgInstallTasks {
    fn get_pkg_stack(
        core_db: &Database,
        pkg_to_query: PkgToQuery,
    ) -> Result<Vec<PkgIndex>, LpmError<MainError>>;
    fn start_install_task(
        core_db: &Database,
        path: &str,
        src_pkg_id: Option<i64>,
    ) -> Result<(), LpmError<MainError>>;
    fn copy_programs(&self) -> Result<(), LpmError<MainError>>;
    fn copy_scripts(&self) -> Result<(), LpmError<MainError>>;
    fn install(&self) -> Result<(), LpmError<MainError>>;
}

impl PkgInstallTasks for PkgDataFromFs {
    /// Finds package dependencies and returns it with the package it self.
    fn get_pkg_stack(
        core_db: &Database,
        pkg_to_query: PkgToQuery,
    ) -> Result<Vec<PkgIndex>, LpmError<MainError>> {
        let index_db_list = db::get_repositories(core_db)?;
        if index_db_list.is_empty() {
            info!("No repository has been found within the database.");
            return Err(
                RepositoryErrorKind::PackageNotFound(pkg_to_query.name).to_lpm_err(),
            )?;
        }

        let index = find_pkg_index(&index_db_list, &pkg_to_query)?;

        let mut pkg_stack = vec![index];
        for (name, repository_address) in index_db_list {
            let repository_db_path = Path::new(db::REPOSITORY_INDEX_DB_DIR).join(&name);
            let db_file = fs::metadata(&repository_db_path)?;
            let index_db = Database::open(Path::new(&repository_db_path))?;
            let is_initialized = db_file.len() > 0;

            if !is_initialized {
                warning!("{name} repository is not initialized");
                continue;
            }

            let mut i = 0;
            loop {
                if i >= pkg_stack.len() {
                    break;
                }

                let pkg = &pkg_stack[i];
                let pkg_name = format!(
                    "{}@{}{}",
                    pkg.name,
                    pkg.version.condition.to_str_operator(),
                    pkg.version.readable_format
                );

                let pkg_to_query = some_or_error!(
                    PkgToQuery::parse(&pkg_name),
                    "Failed resolving package name '{pkg_name}'"
                );

                let new_pkgs: Vec<PkgIndex> =
                    db::PkgIndex::get_mandatory_dependencies(&index_db, &pkg_to_query)?
                        .iter()
                        .map(|pkg_name| {
                            let pkg_to_query = some_or_error!(
                                PkgToQuery::parse(pkg_name),
                                "Failed resolving package name '{pkg_name}'"
                            );

                            PkgIndex {
                                name: pkg_to_query.name.clone(),
                                repository_address: repository_address.clone(),
                                version: pkg_to_query.version_struct(),
                            }
                        })
                        .collect();

                pkg_stack.extend(new_pkgs);

                i += 1;
            }
        }

        // Do not have same package with multiple versions. Which
        // might happen when same package exists in multiple repositories.
        pkg_stack.dedup_by_key(|t| t.name.clone());

        Ok(pkg_stack)
    }

    fn start_install_task(
        core_db: &Database,
        path: &str,
        src_pkg_id: Option<i64>,
    ) -> Result<(), LpmError<MainError>> {
        let pkg_path = PathBuf::from(path);

        info!("Extracting..");
        let pkg = PkgDataFromFs::start_extract_task(&pkg_path)?;

        if is_package_exists(core_db, &pkg.meta_dir.meta.name)? {
            logger::info!(
                "Package '{}' already installed on your machine.",
                pkg.meta_dir.meta.name
            );
            return Ok(());
        }

        info!("Validating files..");
        pkg.start_validate_task()?;

        info!("Syncing with package database..");
        let _pkg_id = pkg.insert_to_db(core_db, src_pkg_id)?;

        if let Err(err) = pkg.scripts.execute_script(ScriptPhase::PreInstall) {
            transaction_op(core_db, Transaction::Rollback)?;
            return Err(err);
        }

        info!("Installing package files into system..");
        if let Err(err) = pkg.install() {
            transaction_op(core_db, Transaction::Rollback)?;
            return Err(err);
        };

        info!("Cleaning temporary files..");
        if let Err(err) = pkg.cleanup() {
            transaction_op(core_db, Transaction::Rollback)?;
            return Err(err)?;
        };

        if let Err(err) = pkg.scripts.execute_script(ScriptPhase::PostInstall) {
            transaction_op(core_db, Transaction::Rollback)?;
            return Err(err);
        }

        if let Err(err) = transaction_op(core_db, Transaction::Commit) {
            transaction_op(core_db, Transaction::Rollback)?;
            return Err(err)?;
        };

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

pub fn install_from_repository(
    core_db: &Database,
    pkg_name: &str,
    src_pkg_id: Option<i64>,
) -> Result<(), LpmError<MainError>> {
    let pkg_to_query = PkgToQuery::parse(pkg_name)
        .ok_or_else(|| PackageErrorKind::InvalidPackageName(pkg_name.to_owned()).to_lpm_err())?;

    if is_package_exists(core_db, &pkg_to_query.name)? {
        logger::info!(
            "Package '{}' already installed on your machine.",
            pkg_to_query.to_string()
        );
        return Ok(());
    }

    // Find package stack(package itself and it's dependencies)
    let pkg_stack = PkgDataFromFs::get_pkg_stack(core_db, pkg_to_query)?;

    let mut thread_handlers = Vec::new();

    for item in pkg_stack {
        let pkg_path = item.pkg_output_path(super::EXTRACTION_OUTPUT_PATH);
        let handler = thread::spawn(move || {
            let core_db = crate::open_core_db_connection()?;
            download_file(&item.pkg_url(), &pkg_path)?;
            let pkg_path_as_string = pkg_path.display().to_string();
            info!("Package installation started for {}", &pkg_path_as_string);
            PkgDataFromFs::start_install_task(&core_db, &pkg_path_as_string, src_pkg_id)
        });

        thread_handlers.push(handler);
    }

    for handler in thread_handlers {
        handler.join().unwrap()?;
    }

    Ok(())
}

pub fn install_from_lod_file(
    core_db: &Database,
    pkg_path: &str,
    src_pkg_id: Option<i64>,
) -> Result<(), LpmError<MainError>> {
    info!("Package installation started for {}", pkg_path);
    PkgDataFromFs::start_install_task(core_db, pkg_path, src_pkg_id)?;

    Ok(())
}
