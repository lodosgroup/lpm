mod delete;
mod extract;
mod install;
mod module;
mod repository;
mod stage1;
mod update;
mod validate;

pub use delete::delete_lod;
pub(crate) use extract::PkgExtractTasks;
pub use install::install_lod;
pub use module::{add_module, delete_modules, print_modules, trigger_lpm_module};
pub use repository::{add_repository, print_repositories};
pub use update::update_lod;

use db::{pkg::insert_pkg_kinds, CORE_DB_PATH};
use ehandle::{lpm::LpmError, MainError};
use logger::{info, success};
use min_sqlite3_sys::prelude::*;
use std::path::Path;

const EXTRACTION_OUTPUT_PATH: &str = "/tmp/lpm";

pub fn configure() -> Result<(), LpmError<MainError>> {
    // create lpm directories under `/var/lib` and `/etc`
    #[cfg(not(debug_assertions))]
    {
        std::fs::create_dir_all(Path::new(db::CORE_DB_PATH).parent().unwrap())?;
        std::fs::create_dir_all(db::REPOSITORY_DB_DIR)?;

        std::fs::create_dir_all(stage1::PKG_SCRIPTS_DIR)?;
    }

    db::migrate_database_tables()?;

    Ok(())
}

pub fn add_pkg_kinds(kinds: &[String]) -> Result<(), LpmError<MainError>> {
    if kinds.is_empty() {
        panic!("At least 1 kind must be provided.");
    }

    let db = Database::open(Path::new(CORE_DB_PATH))?;
    info!("Inserting list of package kinds: {:?}", kinds);
    insert_pkg_kinds(&db, kinds.to_vec())?;
    db.close();
    success!("Operation successfully completed.");

    Ok(())
}

pub fn delete_pkg_kinds(kinds: &[String]) -> Result<(), LpmError<MainError>> {
    if kinds.is_empty() {
        panic!("At least 1 kind must be provided.");
    }

    let db = Database::open(Path::new(CORE_DB_PATH))?;
    info!("Deleting list of package kinds: {:?}", kinds);
    db::pkg::delete_pkg_kinds(&db, kinds.to_vec())?;
    db.close();
    success!("Operation successfully completed.");

    Ok(())
}
