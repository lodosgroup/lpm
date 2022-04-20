use common::{pkg::LodPkg, Files};
use min_sqlite3_sys::prelude::*;
use std::path::Path;

pub trait LodPkgCoreDbOps {
    fn insert(&self, db: &Database);
}

impl<'a> LodPkgCoreDbOps for Files {
    fn insert(&self, db: &Database) {
        let files = &self.0;
        let statement = String::from("SELECT LAST_INSERT_ROWID();");
        let mut sql = db
            .prepare(
                statement,
                None::<Box<dyn FnOnce(SqlitePrimaryResult, String)>>,
            )
            .unwrap();

        if PreparedStatementStatus::FoundRow != sql.execute_prepared() {
            // panic
        }
        let pkg_id = sql.get_data::<i64>(0).unwrap();
        println!("{}", pkg_id);

        for file in files {
            println!("{:?}", file);
            let checksum_id = is_checksum_algorithm_exists(db, &file.checksum_algorithm);

            if checksum_id.is_none() {
                // panic
            }

            let file_path = Path::new(&file.path);

            let statement = String::from(
                "
                    INSERT INTO files
                        (name, absolute_path, checksum, checksum_kind_id, package_id)
                    VALUES
                        (?, ?, ?, ?, ?)",
            );

            let mut sql = db
                .prepare(
                    statement,
                    None::<Box<dyn FnOnce(SqlitePrimaryResult, String)>>,
                )
                .unwrap();

            let status = sql.bind_val(1, file_path.file_name().unwrap().to_str().unwrap());
            println!("Status: {:?}", status);
            let status = sql.bind_val(2, format!("/{}", &file.path));
            println!("Status: {:?}", status);
            let status = sql.bind_val(3, &*file.checksum);
            println!("Status: {:?}", status);
            let status = sql.bind_val(4, checksum_id.unwrap());
            println!("Status: {:?}", status);
            let status = sql.bind_val(5, pkg_id);
            println!("Status: {:?}", status);

            if PreparedStatementStatus::Done != sql.execute_prepared() {
                // sql.kill();
                // panic
                println!("hmm, wtf?");
            }

            sql.kill();
        }
    }
}

impl<'a> LodPkgCoreDbOps for LodPkg<'a> {
    fn insert(&self, db: &Database) {
        db.execute(
            String::from("BEGIN TRANSACTION;"),
            Some(super::simple_error_callback),
        )
        .unwrap();

        let meta = &self.meta_dir.as_ref().unwrap().meta;

        let statement = String::from(
            "
                INSERT INTO packages
                    (name, description, maintainer, repository_id,
                    homepage, depended_package_id, package_kind_id,
                    installed_size, license, v_major, v_minor, v_patch,
                    v_tag, v_readable)
                VALUES
                    (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        );

        let mut sql = db
            .prepare(
                statement,
                None::<Box<dyn FnOnce(SqlitePrimaryResult, String)>>,
            )
            .unwrap();

        sql.bind_val(1, meta.name.clone());
        sql.bind_val(2, meta.description.clone());
        sql.bind_val(3, meta.maintainer.clone());
        sql.bind_val(4, 1_u32); // TODO

        if let Some(homepage) = &meta.homepage {
            sql.bind_val(5, homepage.clone());
        } else {
            sql.bind_val(5, SQLITE_NULL);
        }

        sql.bind_val(6, SQLITE_NULL); // TODO
        sql.bind_val(7, 1_i32); // TODO
        sql.bind_val(8, meta.installed_size as i64);

        if let Some(license) = &meta.license {
            sql.bind_val(9, license.clone());
        } else {
            sql.bind_val(9, SQLITE_NULL);
        }

        sql.bind_val(10, self.version.major);
        sql.bind_val(11, self.version.minor);
        sql.bind_val(12, self.version.patch);

        if let Some(vtag) = &self.version.tag {
            sql.bind_val(13, vtag.clone());
        } else {
            sql.bind_val(13, SQLITE_NULL);
        }

        sql.bind_val(14, self.version.readable_format.clone());

        if PreparedStatementStatus::Done != sql.execute_prepared() {
            // sql.kill();
            // panic
        }

        self.meta_dir.as_ref().unwrap().files.insert(db);

        sql.kill();

        db.execute(String::from("COMMIT;"), Some(super::simple_error_callback))
            .unwrap();
    }
}

pub fn is_checksum_algorithm_exists(db: &Database, algorithm: &str) -> Option<i64> {
    let statement = String::from("SELECT id FROM checksum_kinds WHERE kind = ?;");

    let mut sql = db
        .prepare(
            statement,
            None::<Box<dyn FnOnce(SqlitePrimaryResult, String)>>,
        )
        .unwrap();

    sql.bind_val(1, algorithm);

    if let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
        let result = sql.get_data::<i64>(0).unwrap();
        return Some(result);
    }

    None
}

pub fn insert_pkg_kinds(
    kinds: Vec<String>,
    db: &Database,
) -> Result<SqlitePrimaryResult, MinSqliteWrapperError> {
    db.execute(
        String::from("BEGIN TRANSACTION;"),
        Some(super::simple_error_callback),
    )?;

    for kind in kinds {
        let statement = String::from(
            "
                INSERT INTO package_kinds
                    (kind)
                VALUES
                    (?);",
        );

        let mut sql = db.prepare(statement, Some(super::simple_error_callback))?;

        sql.bind_val(1, kind);

        sql.execute_prepared();
    }

    db.execute(String::from("COMMIT;"), Some(super::simple_error_callback))
}

pub fn delete_pkg_kinds(
    kinds: Vec<String>,
    db: &Database,
) -> Result<SqlitePrimaryResult, MinSqliteWrapperError> {
    db.execute(
        String::from("BEGIN TRANSACTION;"),
        Some(super::simple_error_callback),
    )?;

    for kind in kinds {
        let statement = String::from(
            "
                DELETE FROM package_kinds
                WHERE
                    kind = ?;",
        );

        let mut sql = db.prepare(statement, Some(super::simple_error_callback))?;
        sql.bind_val(1, kind);

        sql.execute_prepared();
    }

    db.execute(String::from("COMMIT;"), Some(super::simple_error_callback))
}
