use std::process;

use ehandle::db::MigrationError;
use migrations::start_db_migrations;
use min_sqlite3_sys::prelude::SqlitePrimaryResult;

mod migrations;
pub mod pkg;

#[cfg(not(debug_assertions))]
pub const DB_PATH: &str = "/var/lib/lodpm/lpm.db";

#[cfg(debug_assertions)]
pub const DB_PATH: &str = "lpm.db";

#[inline]
pub fn init_db() -> Result<(), MigrationError> {
    start_db_migrations()
}

#[inline]
pub fn simple_error_callback(status: SqlitePrimaryResult, sql_statement: String) {
    println!(
        "SQL EXECUTION HAS BEEN FAILED.\n\nReason: {:?}\nStatement: {}",
        status, sql_statement
    );

    process::exit(1);
}
