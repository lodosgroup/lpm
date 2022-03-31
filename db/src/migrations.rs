use ehandle::db::{MigrationError, MigrationErrorKind};
use min_sqlite3_sys::prelude::*;
use std::{path::Path, process};

const INITIAL_VERSION: i64 = 0;

pub fn do_migrations() -> Result<(), MigrationError> {
    let db = Database::open(Path::new(super::DB_PATH))?;
    let mut initial_version: i64 = INITIAL_VERSION;

    create_table_core(&db, &mut initial_version)?;
    create_table_core(&db, &mut initial_version)?;
    create_table_core(&db, &mut initial_version)?;

    db.close();

    Ok(())
}

fn set_migration_version(db: &Database, version: i64) -> Result<(), MigrationError> {
    let statement = String::from(format!("PRAGMA user_version = {};", version));
    let status = db.execute(
        statement,
        None::<Box<dyn FnOnce(SqlitePrimaryResult, String)>>,
    )?;

    if status != SqlitePrimaryResult::Ok {
        return Err(MigrationError::new(MigrationErrorKind::VersionCouldNotSet));
    }

    println!("{:?}", status);

    Ok(())
}

fn can_migrate<'a>(db: &Database, version: i64) -> Result<bool, MinSqliteWrapperError<'a>> {
    let statement = String::from("PRAGMA user_version;");

    let mut sql = db.prepare(
        statement,
        None::<Box<dyn FnOnce(SqlitePrimaryResult, String)>>,
    )?;

    let mut result = false;
    while let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
        let db_user_version = sql.clone().get_data::<i64>(0).unwrap();

        result = version > db_user_version;
    }

    sql.kill();

    Ok(result)
}

fn callback_function(status: SqlitePrimaryResult, sql_statement: String) {
    println!(
        "SQL EXECUTION HAS BEEN FAILED.\n\nReason: {:?}\nStatement: {}",
        status, sql_statement
    );

    process::exit(1);
}

fn create_table_core(db: &Database, version: &mut i64) -> Result<(), MigrationError> {
    *version += 1;
    if !can_migrate(db, *version)? {
        return Ok(());
    }

    // TODO
    // define database structure
    let statement = String::from(
        "
            CREATE TABLE pkg_repositories (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               link TEXT NOT NULL
            );

            CREATE TABLE pkg_architectures (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               architecture TEXT NOT NULL
            );

            CREATE TABLE pkg_kinds (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               kind TEXT NOT NULL
            );
        ",
    );

    db.execute(statement, Some(callback_function))?;

    set_migration_version(db, *version)?;

    Ok(())
}
