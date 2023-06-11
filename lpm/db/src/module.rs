use ehandle::{
    db::SqlError, lpm::LpmError, simple_e_fmt, try_bind_val, try_execute_prepared, ErrorCommons,
};
use min_sqlite3_sys::prelude::*;
use sql_builder::delete::*;
use sql_builder::insert::Insert;
use sql_builder::select::*;
use sql_builder::Column;

pub fn insert_module(
    core_db: &Database,
    name: &str,
    dylib_path: &str,
) -> Result<PreparedStatementStatus, LpmError<SqlError>> {
    const NAME_COL_PRE_ID: usize = 1;
    const DYLIB_PATH_COL_PRE_ID: usize = 2;

    let module_columns = vec![
        Column::new(String::from("name"), NAME_COL_PRE_ID),
        Column::new(String::from("dylib_path"), DYLIB_PATH_COL_PRE_ID),
    ];

    let sql_builder = Insert::new(Some(module_columns), String::from("modules"));

    let statement = sql_builder.to_string();

    let mut sql = core_db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, NAME_COL_PRE_ID, name);
    try_bind_val!(sql, DYLIB_PATH_COL_PRE_ID, dylib_path);

    logger::debug!("Inserting module\n  name: {name}\n  dylib_path: {dylib_path}");
    let status = try_execute_prepared!(sql, simple_e_fmt!("Error on inserting module {name}"));

    Ok(status)
}

pub fn delete_modules(
    core_db: &Database,
    module_names: Vec<String>,
) -> Result<PreparedStatementStatus, LpmError<SqlError>> {
    let mut pre_ids = vec![];
    for (index, _) in module_names.iter().enumerate() {
        pre_ids.push(index + 1);
    }

    let statement = Delete::new(String::from("modules"))
        .where_condition(Where::In(pre_ids, String::from("name")))
        .to_string();

    let mut sql = core_db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    for (index, name) in module_names.iter().enumerate() {
        try_bind_val!(sql, index + 1, &**name);
    }

    let module_names = module_names.join(", ");

    let status = try_execute_prepared!(
        sql,
        simple_e_fmt!("Error on deleting modules '{module_names}'")
    );

    Ok(status)
}

pub fn is_module_exists(core_db: &Database, name: &str) -> Result<bool, LpmError<SqlError>> {
    const NAME_COL_PRE_ID: usize = 1;
    let exists_statement = Select::new(None, String::from("modules"))
        .where_condition(Where::Equal(NAME_COL_PRE_ID, String::from("name")))
        .exists()
        .to_string();

    let mut sql = core_db.prepare(exists_statement.clone(), super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, NAME_COL_PRE_ID, name);

    try_execute_prepared!(
        sql,
        simple_e_fmt!("Select exists query failed. SQL:\n {}", exists_statement)
    );

    let result = sql.get_data::<i64>(0).unwrap_or(0);

    Ok(result == 1)
}

pub fn get_dylib_path_by_name(
    core_db: &Database,
    name: &str,
) -> Result<Option<String>, LpmError<SqlError>> {
    const NAME_COL_PRE_ID: usize = 1;
    let select_statement = Select::new(
        Some(vec![String::from("dylib_path")]),
        String::from("modules"),
    )
    .where_condition(Where::Equal(NAME_COL_PRE_ID, String::from("name")))
    .to_string();

    let mut sql = core_db.prepare(select_statement.clone(), super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, NAME_COL_PRE_ID, name);

    try_execute_prepared!(
        sql,
        simple_e_fmt!("Select id query failed. SQL:\n {}", select_statement)
    );

    let result = sql.get_data::<Option<String>>(0)?;
    Ok(result)
}

pub fn get_modules(core_db: &Database) -> Result<Vec<(String, String)>, LpmError<SqlError>> {
    let select_statement = Select::new(None, String::from("modules")).to_string();

    let mut sql = core_db.prepare(select_statement, super::SQL_NO_CALLBACK_FN)?;

    let mut result = vec![];
    while let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
        result.push((sql.get_data(1)?, sql.get_data(2)?));
    }

    Ok(result)
}
