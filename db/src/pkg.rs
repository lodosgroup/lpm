use core::pkg::LodPkg;
use min_sqlite3_sys::prelude::*;

pub trait LodPkgCoreDbOps {
    fn insert(&self, db: &Database);
}

impl<'a> LodPkgCoreDbOps for LodPkg<'a> {
    fn insert(&self, db: &Database) {
        let statement = String::from("PRAGMA user_version");
        let _status = db
            .execute(
                statement,
                None::<Box<dyn FnOnce(SqlitePrimaryResult, String)>>,
            )
            .unwrap();

        todo!()
    }
}

pub fn insert_pkg_kinds(kinds: Vec<String>, db: &Database) {
    let mut statement = String::from(
        "
            INSERT INTO package_kinds
                (kind)
            VALUES
        ",
    );

    for kind in kinds {
        statement = format!("{} ('{}'),", statement, kind);
    }

    statement.pop();
    statement = format!("{}{}", statement, ";");
    db.execute(statement, Some(super::simple_error_callback))
        .unwrap();
}

pub fn delete_pkg_kind(kind: String, db: &Database) {
    let statement = format!(
        "
            DELETE FROM package_kinds
            WHERE
                kind = '{}'
        ",
        kind,
    );
    db.execute(statement, Some(super::simple_error_callback))
        .unwrap();
}
