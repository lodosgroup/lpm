use ehandle::db::MigrationError;
use migrations::start_db_migrations;

mod migrations;

#[cfg(not(debug_assertions))]
const DB_PATH: &str = "/var/lib/lodpm/lpm.db";

#[cfg(debug_assertions)]
const DB_PATH: &str = "lpm.db";

#[inline]
pub fn init_db() -> Result<(), MigrationError> {
    start_db_migrations()
}
