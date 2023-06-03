use db::{
    get_repositories, insert_repository, is_repository_exists, CORE_DB_PATH, REPOSITORY_DB_DIR,
};
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

pub fn print_repositories() -> Result<(), LpmError<RepositoryError>> {
    let db = Database::open(Path::new(CORE_DB_PATH))?;

    info!("Getting repository list from the database..");
    let list = get_repositories(&db)?;
    db.close();

    println!();

    if list.is_empty() {
        println!("No repository has been found within the database.");
        return Ok(());
    }

    println!("Registered repository list:");
    for item in list {
        println!("  {}: {}", item.0, item.1);
    }

    Ok(())
}
