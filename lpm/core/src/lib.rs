mod delete;
mod extract;
mod install;
mod module;
mod repository;
mod stage1;
mod update;
mod validate;

use std::path::Path;

pub use delete::delete_lod;
pub(crate) use extract::PkgExtractTasks;
pub use install::{install_from_lod_file, install_from_repository};
pub use module::{add_module, delete_modules, print_modules, trigger_lpm_module};
pub use repository::get_and_apply_repository_patches;
pub use repository::{add_repository, delete_repositories, print_repositories};
pub use update::{update_from_lod_file, update_from_repository};

use ehandle::{lpm::LpmError, MainError};
use min_sqlite3_sys::prelude::*;

const EXTRACTION_OUTPUT_PATH: &str = "/tmp/lpm";

pub fn update_database_migrations() -> Result<(), LpmError<MainError>> {
    std::fs::create_dir_all(std::path::Path::new(db::CORE_DB_PATH).parent().unwrap())?;
    std::fs::create_dir_all(db::REPOSITORY_INDEX_DB_DIR)?;
    std::fs::create_dir_all(stage1::PKG_SCRIPTS_DIR)?;

    let core_db = Database::open(Path::new(db::CORE_DB_PATH))?;
    db::migrate_database_tables(&core_db)?;

    Ok(())
}
