use crate::SQL_NO_CALLBACK_FN;

use ehandle::{db::SqlError, lpm::LpmError, simple_e_fmt, try_execute_prepared, ErrorCommons};
use min_sqlite3_sys::prelude::*;

pub fn get_last_index_timestamp(db: &Database) -> Result<u32, LpmError<SqlError>> {
    let statement = String::from(
        "SELECT IFNULL(MAX(index_timestamp), 0) FROM repository ORDER BY index_timestamp DESC LIMIT 1;",
    );
    let mut sql = db.prepare(statement.clone(), SQL_NO_CALLBACK_FN)?;

    try_execute_prepared!(
        sql,
        simple_e_fmt!("Failed executing SQL statement `{}`.", statement)
    );

    let index: Option<u32> = sql.get_data(0)?;
    Ok(index.unwrap_or(0))
}
