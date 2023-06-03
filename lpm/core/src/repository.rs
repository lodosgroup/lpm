use db::{insert_repository, is_repository_exists, CORE_DB_PATH, REPOSITORY_DB_DIR};
use ehandle::{
    lpm::LpmError,
    repository::{RepositoryError, RepositoryErrorKind},
    ErrorCommons,
};
use logger::{info, success};
use min_sqlite3_sys::prelude::*;
use std::path::Path;

pub fn add_repository(name: &str, address: &str) -> Result<(), LpmError<RepositoryError>> {
    let repository_db_path = Path::new(REPOSITORY_DB_DIR).join(name);

    let db = Database::open(Path::new(CORE_DB_PATH))?;

    if is_repository_exists(&db, name)? {
        return Err(RepositoryErrorKind::RepositoryAlreadyExists(name.to_owned()).to_lpm_err());
    }

    info!("Adding {name} repository to the database..");
    insert_repository(
        &db,
        name,
        address,
        repository_db_path.to_str().unwrap(),
        true,
    )?;
    db.close();

    info!("Initializing {name} database file..");
    let db = Database::open(repository_db_path)?;
    db.close();

    success!("Operation successfully completed.");

    Ok(())
}
