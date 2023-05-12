const EXTRACTION_OUTPUT_PATH: &str = "/tmp/lpm";

mod delete;
mod extract;
mod install;
mod module;
mod update;
mod validate;

pub use delete::delete_lod;
pub(crate) use extract::PkgExtractTasks;
pub use install::install_lod;
pub use module::trigger_lpm_module;
pub use update::update_lod;

use common::{config::create_default_config_file, log_and_panic};
use db::{pkg::insert_pkg_kinds, DB_PATH};
use ehandle::{lpm::LpmError, MainError};
use logger::{info, success};
use min_sqlite3_sys::prelude::*;
use std::path::Path;

pub fn configure() -> Result<(), LpmError<MainError>> {
    // create lpm directories under `/var/lib` and `/etc`
    #[cfg(not(debug_assertions))]
    {
        // creating `pkg` dir is already enough, but it's nice to have this
        // in the codebase explicitly.
        std::fs::create_dir_all("/var/lib/lpm")?;
        std::fs::create_dir_all("/var/lib/lpm/pkg")?;
        std::fs::create_dir_all("/etc/lpm")?;
    }

    create_default_config_file()?;
    db::migrate_database_tables()?;

    Ok(())
}

pub fn add_pkg_kinds(kinds: &[String]) -> Result<(), LpmError<MainError>> {
    if kinds.is_empty() {
        log_and_panic!("At least 1 kind must be provided.");
    }

    let db = Database::open(Path::new(DB_PATH))?;
    info!("Inserting list of package kinds: {:?}", kinds);
    insert_pkg_kinds(&db, kinds.to_vec())?;
    db.close();
    success!("Operation successfully completed.");

    Ok(())
}

pub fn delete_pkg_kinds(kinds: &[String]) -> Result<(), LpmError<MainError>> {
    if kinds.is_empty() {
        log_and_panic!("At least 1 kind must be provided.");
    }

    let db = Database::open(Path::new(DB_PATH))?;
    info!("Deleting list of package kinds: {:?}", kinds);
    db::pkg::delete_pkg_kinds(&db, kinds.to_vec())?;
    db.close();
    success!("Operation successfully completed.");

    Ok(())
}
