use ehandle::{
    db::{SqlError, SqlErrorKind},
    lpm::LpmError,
    simple_e_fmt, try_execute_prepared, ErrorCommons,
};
use min_sqlite3_sys::prelude::*;

pub use index::PkgIndex;
pub use migrations::migrate_database_tables;
pub use module::{
    delete_modules, get_dylib_path_by_name, get_modules, insert_module, is_module_exists,
};
pub use repository::{
    delete_repositories, get_repositories, insert_repository, is_repository_exists,
};

pub const REPOSITORY_INDEX_DB_DIR: &str = "/var/lib/lpm/db/repositories";
pub const CORE_DB_PATH: &str = "/var/lib/lpm/db/core-db";

pub const SQL_NO_CALLBACK_FN: Option<
    Box<dyn FnOnce(min_sqlite3_sys::bindings::SqlitePrimaryResult, String)>,
> = None::<Box<dyn FnOnce(SqlitePrimaryResult, String)>>;

#[allow(clippy::disallowed_methods)]
pub fn enable_foreign_keys(any_db: &Database) -> Result<(), LpmError<SqlError>> {
    any_db.execute(
        String::from("PRAGMA foreign_keys = on;"),
        SQL_NO_CALLBACK_FN,
    )?;

    Ok(())
}

#[allow(clippy::disallowed_methods)]
pub fn enable_core_db_pragmas(core_db: &Database) -> Result<(), LpmError<SqlError>> {
    core_db.execute(
        String::from("PRAGMA journal_mode = DELETE;"),
        SQL_NO_CALLBACK_FN,
    )?;

    core_db.execute(
        String::from("PRAGMA journal_mode = WAL2;"),
        SQL_NO_CALLBACK_FN,
    )?;

    core_db.execute(
        String::from("PRAGMA temp_storage = memory;"),
        SQL_NO_CALLBACK_FN,
    )?;

    core_db.execute(
        String::from("PRAGMA synchronous = normal;"),
        SQL_NO_CALLBACK_FN,
    )?;

    Ok(())
}

#[allow(clippy::disallowed_methods)]
pub fn enable_core_db_wal1(core_db: &Database) -> Result<(), LpmError<SqlError>> {
    core_db.execute(
        String::from("PRAGMA journal_mode = WAL;"),
        SQL_NO_CALLBACK_FN,
    )?;

    Ok(())
}

fn get_last_insert_row_id(any_db: &Database) -> Result<i64, LpmError<SqlError>> {
    let statement = String::from("SELECT LAST_INSERT_ROWID();");
    let mut sql = any_db.prepare(statement.clone(), SQL_NO_CALLBACK_FN)?;

    try_execute_prepared!(
        sql,
        simple_e_fmt!("Failed executing SQL statement `{}`.", statement)
    );

    let data = sql.get_data::<i64>(0)?;
    Ok(data)
}

pub enum Transaction {
    Begin,
    Commit,
    Rollback,
}

impl Transaction {
    fn to_statement(&self) -> String {
        match self {
            Transaction::Begin => String::from("BEGIN;"),
            Transaction::Commit => String::from("COMMIT;"),
            Transaction::Rollback => String::from("ROLLBACK;"),
        }
    }
}

pub fn transaction_op(
    any_db: &Database,
    transaction: Transaction,
) -> Result<SqlitePrimaryResult, LpmError<SqlError>> {
    #[allow(clippy::disallowed_methods)]
    match any_db.execute(transaction.to_statement(), SQL_NO_CALLBACK_FN)? {
        SqlitePrimaryResult::Ok => Ok(SqlitePrimaryResult::Ok),
        e => Err(SqlErrorKind::FailedExecuting(transaction.to_statement(), e).to_lpm_err()),
    }
}

pub fn get_current_datetime(any_db: &Database) -> Result<String, LpmError<SqlError>> {
    let statement = String::from("SELECT datetime(CURRENT_TIMESTAMP, 'localtime');");
    let mut sql = any_db.prepare(statement.clone(), SQL_NO_CALLBACK_FN)?;

    try_execute_prepared!(
        sql,
        simple_e_fmt!("Failed executing SQL statement `{}`.", statement)
    );

    let data = sql.get_data::<String>(0)?;
    Ok(data)
}

mod index;
mod migrations;
mod module;
pub mod pkg;
mod repository;
