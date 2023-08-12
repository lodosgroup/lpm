mod ctx;
mod delete;
mod extract;
mod install;
mod module;
mod repository;
mod stage1;
mod update;
mod validate;

use db::enable_core_db_pragmas;
use std::path::Path;

pub use ctx::Ctx;
pub use delete::delete_lod;
pub(crate) use extract::PkgExtractTasks;
pub use install::{install_from_lod_file, install_from_repository};
pub use module::{add_module, delete_modules, print_modules, trigger_lpm_module};
pub use repository::get_and_apply_repository_patches;
pub use repository::{add_repository, delete_repositories, print_repositories};
pub use update::{
    update_pkg_from_lod_file, update_pkg_from_repository, update_pkgs_from_repository,
};

use ehandle::{lpm::LpmError, MainError};
use min_sqlite3_sys::prelude::*;

const EXTRACTION_OUTPUT_PATH: &str = "/tmp/lpm";

pub fn update_database_migrations() -> Result<(), LpmError<MainError>> {
    std::fs::create_dir_all(std::path::Path::new(db::CORE_DB_PATH).parent().unwrap())?;
    std::fs::create_dir_all(db::REPOSITORY_INDEX_DB_DIR)?;
    std::fs::create_dir_all(stage1::PKG_SCRIPTS_DIR)?;

    db::migrate_database_tables(&open_core_db_connection()?)?;

    Ok(())
}

pub fn open_core_db_connection() -> Result<Database, LpmError<MainError>> {
    let core_db = Database::open(Path::new(db::CORE_DB_PATH))?;
    enable_core_db_pragmas(&core_db)?;
    Ok(core_db)
}
