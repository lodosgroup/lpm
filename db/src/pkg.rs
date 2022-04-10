use core::pkg::LodPkg;
use min_sqlite3_sys::prelude::*;

pub trait LodPkgCoreDbOps {
    fn insert(&self, db: &Database);
}

impl<'a> LodPkgCoreDbOps for LodPkg<'a> {
    fn insert(&self, db: &Database) {
        let statement = String::from(";");
        let _status = db
            .execute(
                statement,
                None::<Box<dyn FnOnce(SqlitePrimaryResult, String)>>,
            )
            .unwrap();

        todo!()
    }
}

pub fn insert_pkg_kinds(
    kinds: Vec<String>,
    db: &Database,
) -> Result<SqlitePrimaryResult, MinSqliteWrapperError> {
    let mut statement = String::from(
        "
            INSERT INTO package_kinds
                (kind)
            VALUES",
    );

    for kind in kinds {
        statement = format!("{} ('{}'),", statement, kind);
    }

    statement.pop();
    statement = format!("{}{}", statement, ";");
    db.execute(statement, Some(super::simple_error_callback))
}

pub fn delete_pkg_kind(
    kinds: Vec<String>,
    db: &Database,
) -> Result<SqlitePrimaryResult, MinSqliteWrapperError> {
    let mut statement = String::from(
        "
            DELETE FROM package_kinds
            WHERE
                kind IN (",
    );

    for kind in kinds {
        statement = format!("{} '{}',", statement, kind);
    }

    statement.pop();
    statement = format!("{}){}", statement, ";");

    db.execute(statement, Some(super::simple_error_callback))
}
