use common::{pkg::LodPkg, Files};
use ehandle::{
    db::SqlError,
    pkg::{PackageError, PackageErrorKind},
    ErrorCommons,
};
use min_sqlite3_sys::prelude::*;
use std::path::Path;

pub trait LodPkgCoreDbOps {
    fn insert(&self, db: &Database) -> Result<(), PackageError>;
}

// Maybe don't implement LodPkgCoreDbOps, since
// the insert functionality very coupled with `LodPkg::insert`
// and should be private for this module.
impl<'a> LodPkgCoreDbOps for Files {
    fn insert(&self, db: &Database) -> Result<(), PackageError> {
        let files = &self.0;

        let pkg_id = super::get_last_insert_row_id(db)?;

        for file in files {
            let checksum_id = get_checksum_algorithm_id_by_kind(db, &file.checksum_algorithm)?;
            if checksum_id.is_none() {
                return Err(PackageErrorKind::UnsupportedChecksumAlgorithm(Some(format!(
                    "{} algorithm is not supported from current lpm version.",
                    &file.checksum_algorithm
                )))
                .throw());
            }

            let file_path = Path::new(&file.path);

            let statement = String::from(
                "
                    INSERT INTO files
                        (name, absolute_path, checksum, checksum_kind_id, package_id)
                    VALUES
                        (?, ?, ?, ?, ?)",
            );

            let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

            // TODO
            // Remove these debug lines and handle them via custom macro provided for this job
            // before merging to stable
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

            // TODO
            // provide macro for `sql.execute_prepared()` to handle errors
            if PreparedStatementStatus::Done != sql.execute_prepared() {
                sql.kill();
                return Err(PackageErrorKind::InstallationFailed(None).throw());
            }

            sql.kill();
        }

        Ok(())
    }
}

impl<'a> LodPkgCoreDbOps for LodPkg<'a> {
    fn insert(&self, db: &Database) -> Result<(), PackageError> {
        let meta = &self.meta_dir.as_ref().unwrap().meta;

        if is_package_exists(db, &meta.name)? {
            return Err(PackageErrorKind::AlreadyInstalled(Some(format!(
                "{} is already installed in your system.",
                meta.name
            )))
            .throw());
        }

        db.execute(
            String::from("BEGIN TRANSACTION;"),
            Some(super::simple_error_callback),
        )?;

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

        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

        sql.bind_val(1, meta.name.clone());
        sql.bind_val(2, meta.description.clone());
        sql.bind_val(3, meta.maintainer.clone());
        sql.bind_val(4, SQLITE_NULL); // TODO

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

        // TODO
        // provide macro for `sql.execute_prepared()` to handle errors
        if PreparedStatementStatus::Done != sql.execute_prepared() {
            sql.kill();
            db.execute(
                String::from("ROLLBACK;"),
                Some(super::simple_error_callback),
            )?;
            return Err(PackageErrorKind::InstallationFailed(None).throw());
        }

        sql.kill();

        match self.meta_dir.as_ref().unwrap().files.insert(db) {
            Ok(_) => (),
            Err(err) => {
                db.execute(
                    String::from("ROLLBACK;"),
                    Some(super::simple_error_callback),
                )?;
                return Err(err);
            }
        };

        match db.execute(String::from("COMMIT;"), Some(super::simple_error_callback)) {
            Ok(_) => Ok(()),
            Err(err) => {
                db.execute(
                    String::from("ROLLBACK;"),
                    Some(super::simple_error_callback),
                )?;

                Err(err.into())
            }
        }
    }
}

pub fn is_package_exists(db: &Database, name: &str) -> Result<bool, SqlError> {
    let statement = String::from("SELECT EXISTS(SELECT 1 FROM packages WHERE name = ?);");

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    sql.bind_val(1, name);

    if let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
        let result = sql.get_data::<i64>(0).unwrap_or(0);
        sql.kill();

        return Ok(result == 1);
    }

    sql.kill();
    Ok(false)
}

pub fn get_checksum_algorithm_id_by_kind(
    db: &Database,
    algorithm: &str,
) -> Result<Option<i64>, SqlError> {
    let statement = String::from("SELECT id FROM checksum_kinds WHERE kind = ?;");

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    sql.bind_val(1, algorithm);

    if let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
        let result = sql.get_data::<i64>(0).unwrap();
        sql.kill();
        return Ok(Some(result));
    }

    sql.kill();
    Ok(None)
}

pub fn insert_pkg_kinds(
    kinds: Vec<String>,
    db: &Database,
) -> Result<SqlitePrimaryResult, SqlError> {
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
        sql.kill();
    }

    Ok(db.execute(String::from("COMMIT;"), super::SQL_NO_CALLBACK_FN)?)
}

pub fn delete_pkg_kinds(
    kinds: Vec<String>,
    db: &Database,
) -> Result<SqlitePrimaryResult, SqlError> {
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
        sql.kill();
    }

    Ok(db.execute(String::from("COMMIT;"), super::SQL_NO_CALLBACK_FN)?)
}
