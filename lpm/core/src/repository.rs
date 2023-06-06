use db::{
    get_repositories, insert_repository, is_repository_exists, PkgIndex, CORE_DB_PATH,
    REPOSITORY_DB_DIR, SQL_NO_CALLBACK_FN,
};
use ehandle::{
    lpm::LpmError,
    repository::{RepositoryError, RepositoryErrorKind},
    ErrorCommons,
};
use logger::{debug, info, success};
use min_sqlite3_sys::prelude::*;
use rekuest::Rekuest;
use std::{fs, path::Path};

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

pub fn delete_repositories(repository_names: &[String]) -> Result<(), LpmError<RepositoryError>> {
    if repository_names.is_empty() {
        panic!("At least 1 repository must be provided.");
    }

    let db = Database::open(Path::new(CORE_DB_PATH))?;

    for name in repository_names {
        if !is_repository_exists(&db, name)? {
            return Err(RepositoryErrorKind::RepositoryNotFound(name.to_owned()).to_lpm_err());
        }
    }

    info!("Deleting list of repositories: {:?}", repository_names);
    db::delete_repositories(&db, repository_names.to_vec())?;
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

pub fn get_and_apply_repository_patches() -> Result<(), LpmError<RepositoryError>> {
    let db = Database::open(Path::new(CORE_DB_PATH))?;

    info!("Getting repository list from the database..");
    let list = get_repositories(&db)?;
    db.close();

    if list.is_empty() {
        info!("No repository has been found within the database.");
        return Ok(());
    }

    for (name, address) in &list {
        let repository_db_path = Path::new(REPOSITORY_DB_DIR).join(name);
        let db = Database::open(Path::new(&repository_db_path))?;

        let db_file = fs::metadata(&repository_db_path)?;
        let index_timestamp = if db_file.len() == 0 {
            0
        } else {
            PkgIndex::latest_timestamp(&db)?
        };

        let req_url = format!("{address}/index-tracker/{index_timestamp}");
        debug!("Sending request to '{req_url}'");
        let r = Rekuest::new(&req_url)?.get()?;
        let patch = String::from_utf8(r.body)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        debug!("Applying:\n\n {patch}");

        if !patch.is_empty() {
            #[allow(clippy::disallowed_methods)]
            db.execute(patch, SQL_NO_CALLBACK_FN)?;
        }

        info!("Index of '{name}' is successfully updated.");

        db.close();
    }

    Ok(())
}

pub(crate) fn find_most_recent_pkg_index(
    pkg_name: &str,
) -> Result<PkgIndex, LpmError<RepositoryError>> {
    let db = Database::open(Path::new(CORE_DB_PATH))?;

    let list = get_repositories(&db)?;
    db.close();

    if list.is_empty() {
        info!("No repository has been found within the database.");
        return Err(RepositoryErrorKind::PackageNotFound(pkg_name.to_owned()).to_lpm_err());
    }

    let mut most_recent_index = PkgIndex::default();

    for (name, address) in &list {
        let repository_db_path = Path::new(REPOSITORY_DB_DIR).join(name);
        let db = Database::open(Path::new(&repository_db_path))?;

        if let Some(index) =
            PkgIndex::get_by_pkg_name(&db, pkg_name.to_owned(), address.to_owned())?
        {
            if index.version.compare(&most_recent_index.version) == std::cmp::Ordering::Greater {
                most_recent_index = index
            };
        }
    }

    if most_recent_index.version.readable_format.is_empty() {
        return Err(RepositoryErrorKind::PackageNotFound(pkg_name.to_owned()).to_lpm_err());
    }

    Ok(most_recent_index)
}
