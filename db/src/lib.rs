pub mod migrations;

#[cfg(not(debug_assertions))]
const DB_PATH: &str = "/var/lib/lodpm/lpm.db";

#[cfg(debug_assertions)]
const DB_PATH: &str = "lpm.db";
