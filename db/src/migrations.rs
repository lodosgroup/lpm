use core::get_lpm_version;
use ehandle::db::{MigrationError, MigrationErrorKind};
use min_sqlite3_sys::prelude::*;
use std::path::Path;

const INITIAL_VERSION: i64 = 0;

pub(crate) fn start_db_migrations() -> Result<(), MigrationError> {
    let db = Database::open(Path::new(super::DB_PATH))?;
    let mut initial_version: i64 = INITIAL_VERSION;

    create_table_core(&db, &mut initial_version)?;
    insert_defaults(&db, &mut initial_version)?;

    db.close();

    Ok(())
}

#[inline]
fn set_migration_version(db: &Database, version: i64) -> Result<(), MigrationError> {
    let statement = format!("PRAGMA user_version = {};", version);
    let status = db.execute(
        statement,
        None::<Box<dyn FnOnce(SqlitePrimaryResult, String)>>,
    )?;

    if status != SqlitePrimaryResult::Ok {
        return Err(MigrationError::new(MigrationErrorKind::VersionCouldNotSet));
    }

    Ok(())
}

#[inline]
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

fn create_table_core(db: &Database, version: &mut i64) -> Result<(), MigrationError> {
    *version += 1;
    if !can_migrate(db, *version)? {
        return Ok(());
    }

    let statement = String::from(
        "
            PRAGMA foreign_keys = on;

            /*
             * Statement of `sys` table creation.
             * This table will hold the core informations about lpm.
            */
            CREATE TABLE sys (
               id            INTEGER    PRIMARY KEY    AUTOINCREMENT,
               name          TEXT       NOT NULL       UNIQUE,
               v_major       INTEGER    NOT NULL,
               v_minor       INTEGER    NOT NULL,
               v_patch       INTEGER    NOT NULL,
               v_tag         TEXT,
               v_readable    TEXT       NOT NULL
            );

            /*
             * Statement of `checksum_kinds` table creation.
             * This table will hold the supported hashing algorithms
             * for the packages.
            */
            CREATE TABLE checksum_kinds (
               id      INTEGER    PRIMARY KEY    AUTOINCREMENT,
               kind    TEXT       NOT NULL       UNIQUE
            );

            /*
             * Statement of `package_kinds` table creation.
             * This table will hold the kind of packages to help
             * classify the packages installed in the system.
            */
            CREATE TABLE package_kinds (
               id      INTEGER    PRIMARY KEY    AUTOINCREMENT,
               kind    TEXT       NOT NULL       UNIQUE
            );

            /*
             * Statement of `package_repositories` table creation.
             * This table will hold the repository informations.
            */
            CREATE TABLE package_repositories (
               id            INTEGER    PRIMARY KEY    AUTOINCREMENT,
               repository    TEXT       NOT NULL       UNIQUE
            );

            /*
             * Statement of `packages` table creation.
             * This table will hold installed package informations.
            */
            CREATE TABLE packages (
               id                       INTEGER    PRIMARY KEY    AUTOINCREMENT,
               name                     TEXT       NOT NULL       UNIQUE,
               description              TEXT,
               maintainer               TEXT       NOT NULL,
               repository_id            INTEGER,
               homepage                 TEXT,
               depended_package_id      INTEGER,
               package_kind_id          INTEGER    NOT_NULL,
               installed_size           INTEGER    NOT_NULL,
               license                  TEXT       NOT_NULL,
               v_major                  INTEGER    NOT NULL,
               v_minor                  INTEGER    NOT NULL,
               v_patch                  INTEGER    NOT NULL,
               v_tag                    TEXT,
               v_readable               TEXT       NOT NULL,

               FOREIGN KEY(repository_id) REFERENCES package_repositories(id),
               FOREIGN KEY(depended_package_id) REFERENCES packages(id),
               FOREIGN KEY(package_kind_id) REFERENCES package_kinds(id)
            );

            /*
             * Statement of `files` table creation.
             * This table will hold the information of files which are in the
             * packages.
            */
            CREATE TABLE files (
               id                  INTEGER    PRIMARY KEY    AUTOINCREMENT,
               name                TEXT       NOT NULL,
               absolute_path       TEXT       NOT NULL       UNIQUE,
               checksum            TEXT       NOT NULL,
               checksum_kind_id    INTEGER    NOT NULL,
               package_id          INTEGER    NOT NULL,
               FOREIGN KEY(package_id) REFERENCES packages(id),
               FOREIGN KEY(checksum_kind_id) REFERENCES checksum_kinds(id)
            );
        ",
    );

    db.execute(statement, Some(super::simple_error_callback))?;

    set_migration_version(db, *version)?;

    Ok(())
}

fn insert_defaults(db: &Database, version: &mut i64) -> Result<(), MigrationError> {
    *version += 1;
    if !can_migrate(db, *version)? {
        return Ok(());
    }

    let lpm_version = get_lpm_version();

    let sys_defaults = format!(
        "
            INSERT INTO sys
                (name, v_major, v_minor, v_patch, v_readable)
            VALUES
                ('lpm', {}, {}, {}, '{}');",
        lpm_version.major, lpm_version.minor, lpm_version.patch, lpm_version.readable_format
    );

    let checksum_kind_defaults = String::from(
        "
            INSERT INTO checksum_kinds
                (kind)
            VALUES
                ('md5'),
                ('sha256'),
                ('sha512');",
    );

    let statement = format!(
        "
            {}

            {}
        ",
        sys_defaults, checksum_kind_defaults
    );

    db.execute(statement, Some(super::simple_error_callback))?;

    set_migration_version(db, *version)?;

    Ok(())
}
