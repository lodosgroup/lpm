use ehandle::{
    db::{MigrationError, SqlError, SqlErrorKind},
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

    let data = sql.get_data::<i64>(0).unwrap();
    sql.kill();
    Ok(data)
}

pub enum Transaction {
    Begin,
    Commit,
    Rollback,
}

impl Transaction {
    #[inline(always)]
    fn to_statement(&self) -> &str {
        match self {
            Transaction::Begin => "BEGIN;",
            Transaction::Commit => "COMMIT;",
            Transaction::Rollback => "ROLLBACK;",
        }
    }
}

pub fn transaction_op(
    db: &Database,
    transaction: Transaction,
) -> Result<SqlitePrimaryResult, SqlError> {
    let statement = transaction.to_statement();
    #[allow(clippy::disallowed_methods)]
    match db.execute(statement.to_owned(), SQL_NO_CALLBACK_FN)? {
        SqlitePrimaryResult::Ok => Ok(SqlitePrimaryResult::Ok),
        _ => {
            return Err(SqlErrorKind::FailedExecuting(Some(simple_e_fmt!(
                "Failed executing SQL statement `{}`.",
                statement
            )))
            .throw());
        }
    }
}

pub mod pkg;
