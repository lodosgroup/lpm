use ehandle::{
    db::SqlError, lpm::LpmError, simple_e_fmt, try_bind_val, try_execute_prepared, ErrorCommons,
};
use min_sqlite3_sys::prelude::*;
use sql_builder::delete::*;
use sql_builder::insert::Insert;
use sql_builder::select::Select;
use sql_builder::Column;

pub fn insert_repository(
    db: &Database,
    name: &str,
    address: &str,
    index_db_path: &str,
    is_active: bool,
) -> Result<PreparedStatementStatus, LpmError<SqlError>> {
    const NAME_COL_PRE_ID: usize = 1;
    const ADDRESS_COL_PRE_ID: usize = 2;
    const INDEX_DB_PATH_COL_PRE_ID: usize = 3;
    const IS_ACTIVE_COL_PRE_ID: usize = 4;

    let repository_columns = vec![
        Column::new(String::from("name"), NAME_COL_PRE_ID),
        Column::new(String::from("address"), ADDRESS_COL_PRE_ID),
        Column::new(String::from("index_db_path"), INDEX_DB_PATH_COL_PRE_ID),
        Column::new(String::from("is_active"), IS_ACTIVE_COL_PRE_ID),
    ];

    let sql_builder = Insert::new(Some(repository_columns), String::from("repositories"));

    let statement = sql_builder.to_string();

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, NAME_COL_PRE_ID, name);
    try_bind_val!(sql, ADDRESS_COL_PRE_ID, address);
    try_bind_val!(sql, INDEX_DB_PATH_COL_PRE_ID, index_db_path);
    try_bind_val!(sql, IS_ACTIVE_COL_PRE_ID, is_active as i32);

    logger::debug!("Inserting repository\n  name: {name}\n  address: {address}");
    let status = try_execute_prepared!(sql, simple_e_fmt!("Error on inserting repository {name}"));

    sql.kill();

    Ok(status)
}

pub fn is_repository_exists(db: &Database, name: &str) -> Result<bool, LpmError<SqlError>> {
    const NAME_COL_PRE_ID: usize = 1;
    let exists_statement = Select::new(None, String::from("repositories"))
        .where_condition(Where::Equal(NAME_COL_PRE_ID, String::from("name")))
        .exists()
        .to_string();

    let mut sql = db.prepare(exists_statement.clone(), super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, NAME_COL_PRE_ID, name);

    try_execute_prepared!(
        sql,
        simple_e_fmt!("Select exists query failed. SQL:\n {}", exists_statement)
    );

    let result = sql.get_data::<i64>(0).unwrap_or(0);
    sql.kill();

    Ok(result == 1)
}

pub fn get_repositories(db: &Database) -> Result<Vec<(String, String)>, LpmError<SqlError>> {
    let select_statement = Select::new(None, String::from("repositories")).to_string();

    let mut sql = db.prepare(select_statement, super::SQL_NO_CALLBACK_FN)?;

    let mut result = vec![];
    while let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
        result.push((sql.get_data(1)?, sql.get_data(2)?));
    }

    sql.kill();

    Ok(result)
}
