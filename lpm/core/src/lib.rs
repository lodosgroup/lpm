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
pub use repository::{add_repository, delete_repositories, print_repositories};
pub use update::update_lod;

use ehandle::{lpm::LpmError, MainError};

const EXTRACTION_OUTPUT_PATH: &str = "/tmp/lpm";

pub fn configure() -> Result<(), LpmError<MainError>> {
    // create lpm directories under `/var/lib` and `/etc`
    #[cfg(not(debug_assertions))]
    {
        std::fs::create_dir_all(std::path::Path::new(db::CORE_DB_PATH).parent().unwrap())?;
        std::fs::create_dir_all(db::REPOSITORY_DB_DIR)?;

        std::fs::create_dir_all(stage1::PKG_SCRIPTS_DIR)?;
    }

    db::migrate_database_tables()?;

    Ok(())
}
