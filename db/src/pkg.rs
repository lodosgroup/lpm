use common::pkg::LodPkg;
use min_sqlite3_sys::prelude::*;

pub trait LodPkgCoreDbOps {
    fn insert(&self, db: &Database);
}

impl<'a> LodPkgCoreDbOps for LodPkg<'a> {
    fn insert(&self, db: &Database) {
        let meta = &self.meta_dir.as_ref().unwrap().meta;
        let null_val = String::from("NULL");

        let statement = format!(
            "
            INSERT INTO packages
                (name, description, maintainer, repository_id,
                homepage, depended_package_id, package_kind_id,
                installed_size, license, v_major, v_minor, v_patch,
                v_tag, v_readable)
            VALUES
                ({}, {}, {}, {}, {}, {}, {}, {}, {},
                {}, {}, {}, {}, {})
        ",
            format!("'{}'", meta.name),
            format!("'{}'", meta.description),
            format!("'{}'", meta.maintainer),
            1,
            format!("'{}'", meta.homepage.as_ref().unwrap_or(&null_val.clone())),
            null_val.clone(),
            1,
            meta.installed_size,
            format!("'{}'", meta.license.as_ref().unwrap_or(&null_val.clone())),
            self.version.major,
            self.version.minor,
            self.version.patch,
            format!(
                "'{}'",
                self.version.tag.as_ref().unwrap_or(&null_val.clone())
            ),
            format!("'{}'", self.version.readable_format),
        );
        //        let _status = db
        //            .execute(
        //                statement,
        //                None::<Box<dyn FnOnce(SqlitePrimaryResult, String)>>,
        //            )
        //            .unwrap();
        //

        println!("{}", statement);
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

pub fn delete_pkg_kinds(
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
