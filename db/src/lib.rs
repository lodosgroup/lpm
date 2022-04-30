use ehandle::{
    db::{MigrationError, SqlError},
    simple_e_fmt, try_execute_prepared, ErrorCommons,
};
use migrations::start_db_migrations;
use min_sqlite3_sys::prelude::*;
mod migrations;

#[cfg(not(debug_assertions))]
pub const DB_PATH: &str = "/var/lib/lodpm/lpm.db";

#[cfg(debug_assertions)]
pub const DB_PATH: &str = "lpm.db";

#[inline(always)]
pub fn init_db() -> Result<(), MigrationError> {
    start_db_migrations()
}

pub const SQL_NO_CALLBACK_FN: Option<
    Box<dyn FnOnce(min_sqlite3_sys::bindings::SqlitePrimaryResult, String)>,
> = None::<Box<dyn FnOnce(SqlitePrimaryResult, String)>>;

fn get_last_insert_row_id(db: &Database) -> Result<i64, SqlError> {
    let statement = String::from("SELECT LAST_INSERT_ROWID();");
    let mut sql = db.prepare(statement.clone(), SQL_NO_CALLBACK_FN).unwrap();

    try_execute_prepared!(
        sql,
        Some(simple_e_fmt!(
            "Failed executing SQL statement `{}`.",
            statement
        ))
    );

    Ok(sql.get_data::<i64>(0).unwrap())
}

pub mod pkg;
