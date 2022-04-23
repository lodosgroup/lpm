use ehandle::{
    db::{MigrationError, SqlError, SqlErrorKind},
    ErrorCommons,
};
use migrations::start_db_migrations;
use min_sqlite3_sys::prelude::*;
use std::process;
mod migrations;

#[cfg(not(debug_assertions))]
pub const DB_PATH: &str = "/var/lib/lodpm/lpm.db";

#[cfg(debug_assertions)]
pub const DB_PATH: &str = "lpm.db";

#[inline]
pub fn init_db() -> Result<(), MigrationError> {
    start_db_migrations()
}

#[inline]
// TODO
// remove this and throw `SqlError` instead.
pub fn simple_error_callback(status: SqlitePrimaryResult, sql_statement: String) {
    println!(
        "SQL EXECUTION HAS BEEN FAILED.\n\nReason: {:?}\nStatement: {}",
        status, sql_statement
    );

    process::exit(1);
}

#[inline]
fn get_last_insert_row_id(db: &Database) -> Result<i64, SqlError> {
    let statement = String::from("SEuLECT LAST_INSERT_ROWID();");
    let mut sql = db
        .prepare(
            statement.clone(),
            None::<Box<dyn FnOnce(SqlitePrimaryResult, String)>>,
        )
        .unwrap();

    if PreparedStatementStatus::FoundRow != sql.execute_prepared() {
        sql.kill();
        return Err(SqlErrorKind::FailedExecuting(Some(format!(
            "Failed executing '{}'",
            statement
        )))
        .throw());
    }

    Ok(sql.get_data::<i64>(0).unwrap())
}

pub mod pkg;
