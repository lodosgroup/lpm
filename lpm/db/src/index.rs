use crate::SQL_NO_CALLBACK_FN;

use common::version::VersionStruct;
use ehandle::{
    db::SqlError, lpm::LpmError, simple_e_fmt, try_bind_val, try_execute_prepared, ErrorCommons,
};
use min_sqlite3_sys::prelude::*;
use sql_builder::select::*;
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct PkgIndex {
    pub name: String,
    pub repository_address: String,
    pub version: VersionStruct,
}

impl PkgIndex {
    pub fn latest_timestamp(db: &Database) -> Result<u32, LpmError<SqlError>> {
        let cols = vec![String::from("IFNULL(MAX(index_timestamp), 0)")];
        let statement = Select::new(Some(cols), String::from("repository"))
            .add_arg(SelectArg::OrderBy(vec![OrderType::Desc(String::from(
                "index_timestamp",
            ))]))
            .add_arg(SelectArg::Limit(1))
            .to_string();

        let mut sql = db.prepare(statement.clone(), SQL_NO_CALLBACK_FN)?;

        try_execute_prepared!(
            sql,
            simple_e_fmt!("Failed executing SQL statement `{}`.", statement)
        );

        let index: Option<u32> = sql.get_data(0)?;
        Ok(index.unwrap_or(0))
    }

    pub fn get_by_pkg_name(
        db: &Database,
        pkg_name: String,
        repository_address: String,
    ) -> Result<Option<Self>, LpmError<SqlError>> {
        const NAME_COL_PRE_ID: usize = 1;

        let columns = vec![
            String::from("v_major"),
            String::from("v_minor"),
            String::from("v_patch"),
            String::from("v_tag"),
            String::from("v_readable"),
        ];

        let statement = Select::new(Some(columns), String::from("repository"))
            .where_condition(Where::Equal(NAME_COL_PRE_ID, String::from("name")))
            .add_arg(SelectArg::OrderBy(vec![
                OrderType::Desc(String::from("v_major")),
                OrderType::Desc(String::from("v_minor")),
                OrderType::Desc(String::from("v_patch")),
            ]))
            .add_arg(SelectArg::Limit(1))
            .to_string();

        let mut sql = db.prepare(statement.clone(), SQL_NO_CALLBACK_FN)?;

        try_bind_val!(sql, NAME_COL_PRE_ID, &*pkg_name);

        let status = try_execute_prepared!(
            sql,
            simple_e_fmt!("Failed executing SQL statement `{}`.", statement)
        );

        if status != PreparedStatementStatus::FoundRow {
            return Ok(None);
        }

        let version = VersionStruct {
            major: sql.get_data(0)?,
            minor: sql.get_data(1)?,
            patch: sql.get_data(2)?,
            tag: sql.get_data(3)?,
            readable_format: sql.get_data(4)?,
        };

        Ok(Some(Self {
            name: pkg_name,
            repository_address,
            version,
        }))
    }

    pub fn pkg_url(&self) -> String {
        format!(
            "{}/{}-{}.lod",
            self.repository_address, self.name, self.version.readable_format
        )
    }

    pub fn pkg_filename(&self) -> String {
        format!("{}-{}.lod", self.name, self.version.readable_format)
    }

    pub fn pkg_output_path(&self, output_dir: &str) -> PathBuf {
        PathBuf::from(output_dir.to_string()).join(self.pkg_filename())
    }
}
