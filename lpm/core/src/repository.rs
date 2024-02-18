use crate::Ctx;

use common::{ctx_confirmation_check, pkg::PkgToQuery};
use db::{
    get_repositories, insert_repository, is_repository_exists, PkgIndex, REPOSITORY_INDEX_DB_DIR,
    SQL_NO_CALLBACK_FN,
};
use ehandle::{
    lpm::LpmError,
    repository::{RepositoryError, RepositoryErrorKind},
    ErrorCommons, MainError,
};
use logger::{debug, info, warning};
use min_sqlite3_sys::prelude::*;
use rekuest::Rekuest;
use std::{fs, path::Path};

pub fn add_repository(ctx: Ctx, name: &str, address: &str) -> Result<(), LpmError<MainError>> {
    let repository_index_db_path = Path::new(REPOSITORY_INDEX_DB_DIR).join(name);

    if is_repository_exists(&ctx.core_db, name)? {
        return Err(RepositoryErrorKind::RepositoryAlreadyExists(name.to_owned()).to_lpm_err())?;
    }

    {
        // TODO
        // use colors
        println!("\nRepository list to be registered:");
        println!("  - {name}: {address}");
        println!();
    }
    ctx_confirmation_check!(ctx);

    info!("Adding {name} repository to the database..");
    insert_repository(
        &ctx.core_db,
        name,
        address,
        repository_index_db_path.to_str().unwrap(),
        true,
    )?;

    {
        info!("Getting {name} indexes..");
        let index_db = Database::open(&repository_index_db_path)?;

        let index_db_file = fs::metadata(&repository_index_db_path)?;
        let index_timestamp = if index_db_file.len() == 0 {
            0
        } else {
            PkgIndex::latest_timestamp(&index_db)?
        };

        let req_url = format!("{address}/index-tracker/{index_timestamp}");
        debug!("Sending request to '{req_url}'");
        let r = Rekuest::new(&req_url)?.get()?;
        let patch = String::from_utf8(r.body)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        debug!("Applying:\n\n {patch}");

        if !patch.is_empty() {
            #[allow(clippy::disallowed_methods)]
            index_db.execute(patch, SQL_NO_CALLBACK_FN)?;
        }

        info!("{name} indexes successfully updated.");
    }

    Ok(())
}

pub fn delete_repositories(
    ctx: Ctx,
    repository_names: &[String],
) -> Result<(), LpmError<MainError>> {
    if repository_names.is_empty() {
        panic!("At least 1 repository must be provided.");
    }

    for name in repository_names {
        if !is_repository_exists(&ctx.core_db, name)? {
            return Err(RepositoryErrorKind::RepositoryNotFound(name.to_owned()).to_lpm_err())?;
        }
    }

    {
        // TODO
        // use colors
        println!("\nRepository list to be deleted:");
        repository_names.iter().for_each(|repository| {
            println!("  - {repository}");
        });
        println!();
    }
    ctx_confirmation_check!(ctx);

    info!("Deleting list of repositories: {:?}", repository_names);
    db::delete_repositories(&ctx.core_db, repository_names.to_vec())?;
    repository_names.iter().for_each(|repository| {
        if let Err(err) = fs::remove_file(Path::new(REPOSITORY_INDEX_DB_DIR).join(repository)) {
            warning!(
                "Couldn't clean the index database of {}. Reason: {}",
                repository,
                err
            )
        }
    });

    Ok(())
}

pub fn print_repositories(core_db: &Database) -> Result<(), LpmError<RepositoryError>> {
    info!("Getting repository list from the database..");
    let list = get_repositories(core_db)?;

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

pub fn get_and_apply_repository_patches(
    core_db: &Database,
) -> Result<(), LpmError<RepositoryError>> {
    info!("Getting repository list from the database..");
    let list = get_repositories(core_db)?;

    if list.is_empty() {
        info!("No repository has been found within the database.");
        return Ok(());
    }

    for (name, address) in &list {
        let repository_index_db_path = Path::new(REPOSITORY_INDEX_DB_DIR).join(name);
        let index_db = Database::open(Path::new(&repository_index_db_path))?;

        let index_db_file = fs::metadata(&repository_index_db_path)?;
        let index_timestamp = if index_db_file.len() == 0 {
            0
        } else {
            PkgIndex::latest_timestamp(&index_db)?
        };

        let req_url = format!("{address}/index-tracker/{index_timestamp}");
        debug!("Sending request to '{req_url}'");
        let r = Rekuest::new(&req_url)?.get()?;
        let patch = String::from_utf8(r.body)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        debug!("Applying:\n\n {patch}");

        if !patch.is_empty() {
            #[allow(clippy::disallowed_methods)]
            index_db.execute(patch, SQL_NO_CALLBACK_FN)?;
        }

        info!("Index of '{name}' is successfully updated.");
    }

    Ok(())
}

/// Finds most recent one when version is not specified
pub(crate) fn find_pkg_index(
    index_db_list: &[(String, String)],
    pkg_to_query: &PkgToQuery,
) -> Result<PkgIndex, LpmError<RepositoryError>> {
    let mut most_recent_index = PkgIndex::default();

    for (name, address) in index_db_list {
        let repository_db_path = Path::new(REPOSITORY_INDEX_DB_DIR).join(name);
        let db_file = fs::metadata(&repository_db_path)?;
        let db = Database::open(Path::new(&repository_db_path))?;
        let is_initialized = db_file.len() > 0;

        if !is_initialized {
            warning!("{name} repository is not initialized");
            continue;
        }

        if let Some(index) =
            PkgIndex::query_pkg_with_versions(&db, pkg_to_query, address.to_owned())?
        {
            if index.version.compare(&most_recent_index.version) == std::cmp::Ordering::Greater {
                most_recent_index = index
            };
        }
    }

    if most_recent_index.version.readable_format.is_empty() {
        return Err(RepositoryErrorKind::PackageNotFound(pkg_to_query.name.clone()).to_lpm_err());
    }

    Ok(most_recent_index)
}
