use crate::{
    extract::{get_pkg_tmp_output_path, PkgExtractTasks},
    repository::find_pkg_index,
    stage1::{Stage1Tasks, PKG_SCRIPTS_DIR},
    validate::PkgValidateTasks,
};

use common::{
    download_file,
    meta::DependencyStruct,
    pkg::{PkgDataFromFs, PkgToQuery, ScriptPhase},
    some_or_error,
};
use db::{
    pkg::{is_package_exists, DbOpsForBuildFile},
    transaction_op, Transaction,
};
use ehandle::{lpm::LpmError, pkg::PackageErrorKind, ErrorCommons, MainError};
use logger::{debug, info, warning};
use min_sqlite3_sys::prelude::*;
use std::{
    fs::{self, create_dir_all, remove_file},
    path::{Path, PathBuf},
    thread,
};

trait PkgInstallTasks {
    fn start_install_task(
        core_db: &Database,
        path: &str,
        src_pkg_id: Option<i64>,
    ) -> Result<(), LpmError<MainError>>;
    fn resolve_dependencies(
        index_db: &Database,
        dependencies: &mut Vec<String>,
    ) -> Result<(), LpmError<MainError>>;
    fn install_dependencies(
        core_db: &Database,
        src_pkg_id: i64,
        pkg_dependencies: &[DependencyStruct],
    ) -> Result<(), LpmError<MainError>>;
    fn copy_programs(&self) -> Result<(), LpmError<MainError>>;
    fn copy_scripts(&self) -> Result<(), LpmError<MainError>>;
    fn install(&self) -> Result<(), LpmError<MainError>>;
}

impl PkgInstallTasks for PkgDataFromFs {
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
        let pkg_id = pkg.insert_to_db(core_db, src_pkg_id)?;

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

        if src_pkg_id.is_none() {
            Self::install_dependencies(core_db, pkg_id, &pkg.meta_dir.meta.dependencies)?;
        }

        info!("Installation transaction completed.");

        Ok(())
    }

    // TODO
    // - remove duplicated dependencies with different versions(e.g. pkg@1.2.3, pkg@1.2.2, pkg@1.3.4)
    //   this can happen when lpm has multiple repositories.
    // - remove dependencies if already installed
    fn resolve_dependencies(
        index_db: &Database,
        dependencies: &mut Vec<String>,
    ) -> Result<(), LpmError<MainError>> {
        let mut i = 0;
        loop {
            if i >= dependencies.len() {
                break;
            }

            let dependency_name = &dependencies[i];
            let pkg_to_query = some_or_error!(
                PkgToQuery::parse(dependency_name),
                "Failed resolving package name '{dependency_name}'"
            );

            let new_dependencies =
                db::PkgIndex::get_mandatory_dependencies(index_db, &pkg_to_query)?;
            dependencies.extend(new_dependencies);
            i += 1;
        }

        dependencies.dedup();

        Ok(())
    }

    fn install_dependencies(
        core_db: &Database,
        src_pkg_id: i64,
        pkg_dependencies: &[DependencyStruct],
    ) -> Result<(), LpmError<MainError>> {
        let mut dependency_stack: Vec<String> = pkg_dependencies
            .iter()
            .map(|dependency| {
                format!(
                    "{}@{}{}",
                    dependency.name,
                    dependency.version.condition.to_str_operator(),
                    dependency.version.readable_format
                )
            })
            .collect();

        let list = db::get_repositories(core_db)?;
        for (name, _) in &list {
            let repository_db_path = Path::new(db::REPOSITORY_INDEX_DB_DIR).join(name);
            let db_file = fs::metadata(&repository_db_path)?;
            let index_db = Database::open(Path::new(&repository_db_path))?;
            let is_initialized = db_file.len() > 0;

            if !is_initialized {
                warning!("{name} repository is not initialized");
                continue;
            }

            Self::resolve_dependencies(&index_db, &mut dependency_stack)?;
        }

        let mut thread_handlers = Vec::new();
        let mut dependency_download_paths = Vec::new();

        for dependency in dependency_stack {
            let pkg_to_query = PkgToQuery::parse(&dependency).ok_or_else(|| {
                PackageErrorKind::InvalidPackageName(dependency.to_owned()).to_lpm_err()
            })?;

            if is_package_exists(core_db, &pkg_to_query.name)? {
                logger::info!(
                    "Dependency '{}' already exists on your machine. Skipping it..",
                    &pkg_to_query.name
                );
                return Ok(());
            }

            let index = find_pkg_index(core_db, &pkg_to_query)?;
            let pkg_path = index.pkg_output_path(super::EXTRACTION_OUTPUT_PATH);
            dependency_download_paths.push(pkg_path.clone());

            let handler = thread::spawn(move || download_file(&index.pkg_url(), &pkg_path));

            thread_handlers.push(handler);
        }

        for handler in thread_handlers {
            handler.join().unwrap()?;
        }

        for dependency_path in dependency_download_paths {
            let dependency_path = dependency_path.to_str().unwrap();
            info!("Installing '{}' dependency..", dependency_path);
            install_from_lod_file(core_db, dependency_path, Some(src_pkg_id))?;
        }

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

    let index = find_pkg_index(core_db, &pkg_to_query)?;
    let pkg_path = index.pkg_output_path(super::EXTRACTION_OUTPUT_PATH);

    download_file(&index.pkg_url(), &pkg_path)?;

    let pkg_path_as_string = pkg_path.display().to_string();
    info!("Package installation started for {}", &pkg_path_as_string);
    PkgDataFromFs::start_install_task(core_db, &pkg_path_as_string, src_pkg_id)?;
    remove_file(pkg_path)?;

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
