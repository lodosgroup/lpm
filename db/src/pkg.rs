use common::{pkg::LodPkg, Files};
use ehandle::{
    db::SqlError,
    pkg::{PackageError, PackageErrorKind},
    try_bind_val, try_execute, try_execute_prepared, ErrorCommons,
};
use min_sqlite3_sys::prelude::*;
use std::path::Path;

pub trait LodPkgCoreDbOps {
    fn insert(&self, db: &Database) -> Result<(), PackageError>;
}

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

            try_bind_val!(sql, 1, file_path.file_name().unwrap().to_str().unwrap());
            try_bind_val!(sql, 2, format!("/{}", &file.path));
            try_bind_val!(sql, 3, &*file.checksum);
            try_bind_val!(sql, 4, checksum_id.unwrap());
            try_bind_val!(sql, 5, pkg_id);

            try_execute_prepared!(sql);

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

        try_execute!(db, String::from("BEGIN TRANSACTION;"));

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

        try_bind_val!(sql, 1, &*meta.name);
        try_bind_val!(sql, 2, &*meta.description);
        try_bind_val!(sql, 3, &*meta.maintainer);
        try_bind_val!(sql, 4, SQLITE_NULL); // TODO

        if let Some(homepage) = &meta.homepage {
            try_bind_val!(sql, 5, &**homepage);
        } else {
            try_bind_val!(sql, 5, SQLITE_NULL);
        }

        if let Some(_repository) = &meta.repository {
            // TODO
            // Get repository id by `&**repository`
            try_bind_val!(sql, 6, SQLITE_NULL);
        } else {
            try_bind_val!(sql, 6, SQLITE_NULL);
        }

        try_bind_val!(sql, 7, 1_i32); // TODO
        try_bind_val!(sql, 8, meta.installed_size as i64);

        if let Some(license) = &meta.license {
            try_bind_val!(sql, 9, &**license);
        } else {
            try_bind_val!(sql, 9, SQLITE_NULL);
        }

        try_bind_val!(sql, 10, self.version.major);
        try_bind_val!(sql, 11, self.version.minor);
        try_bind_val!(sql, 12, self.version.patch);

        if let Some(vtag) = &self.version.tag {
            try_bind_val!(sql, 13, &**vtag);
        } else {
            try_bind_val!(sql, 13, SQLITE_NULL);
        }

        try_bind_val!(sql, 14, &*self.version.readable_format);

        if PreparedStatementStatus::Done != sql.execute_prepared() {
            sql.kill();
            try_execute!(db, String::from("ROLLBACK;"));
            return Err(PackageErrorKind::InstallationFailed(None).throw());
        }

        sql.kill();

        match self.meta_dir.as_ref().unwrap().files.insert(db) {
            Ok(_) => (),
            Err(err) => {
                try_execute!(db, String::from("ROLLBACK;"));
                return Err(err);
            }
        };

        match db.execute(String::from("COMMIT;"), super::SQL_NO_CALLBACK_FN) {
            Ok(_) => Ok(()),
            Err(err) => {
                try_execute!(db, String::from("ROLLBACK;"));
                Err(err.into())
            }
        }
    }
}

pub fn is_package_exists(db: &Database, name: &str) -> Result<bool, SqlError> {
    let statement = String::from("SELECT EXISTS(SELECT 1 FROM packages WHERE name = ?);");

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, 1, name);
    try_execute_prepared!(sql);

    let result = sql.get_data::<i64>(0).unwrap_or(0);
    sql.kill();

    Ok(result == 1)
}

pub fn get_checksum_algorithm_id_by_kind(
    db: &Database,
    algorithm: &str,
) -> Result<Option<i64>, SqlError> {
    let statement = String::from("SELECT id FROM checksum_kinds WHERE kind = ?;");

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, 1, algorithm);
    try_execute_prepared!(sql);

    let result = sql.get_data::<i64>(0).unwrap();
    sql.kill();

    Ok(Some(result))
}

pub fn insert_pkg_kinds(
    kinds: Vec<String>,
    db: &Database,
) -> Result<SqlitePrimaryResult, SqlError> {
    try_execute!(db, String::from("BEGIN TRANSACTION;"));

    for kind in kinds {
        let statement = String::from(
            "
                INSERT INTO package_kinds
                    (kind)
                VALUES
                    (?);",
        );

        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, 1, kind);
        try_execute_prepared!(sql);
        sql.kill();
    }

    Ok(try_execute!(db, String::from("COMMIT;")))
}

pub fn delete_pkg_kinds(
    kinds: Vec<String>,
    db: &Database,
) -> Result<SqlitePrimaryResult, SqlError> {
    try_execute!(db, String::from("BEGIN TRANSACTION;"));

    for kind in kinds {
        let statement = String::from(
            "
                DELETE FROM package_kinds
                WHERE
                    kind = ?;",
        );

        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, 1, kind);
        try_execute_prepared!(sql);
        sql.kill();
    }

    Ok(try_execute!(db, String::from("COMMIT;")))
}
