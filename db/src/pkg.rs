use crate::{enable_foreign_keys, transaction_op, Transaction};

use common::{
    meta::{FileStruct, Meta},
    pkg::{LodPkg, MetaDir},
    version::VersionStruct,
    Files,
};
use ehandle::{
    db::SqlError,
    pkg::{PackageError, PackageErrorKind},
    simple_e_fmt, try_bind_val, try_execute_prepared, ErrorCommons,
};
use min_sqlite3_sys::prelude::*;
use std::path::Path;

pub trait LodPkgCoreDbOps {
    fn from_db<'lpkg>(db: &Database, name: &str) -> Result<LodPkg<'lpkg>, PackageError>;
    fn insert(&self, db: &Database) -> Result<(), PackageError>;
    fn delete(db: &Database, name: &str) -> Result<(), PackageError>;
}

impl<'a> LodPkgCoreDbOps for LodPkg<'a> {
    fn insert(&self, db: &Database) -> Result<(), PackageError> {
        enable_foreign_keys(db)?;

        let meta_dir = &self.meta_dir.as_ref().unwrap();

        if is_package_exists(db, &meta_dir.meta.name)? {
            return Err(PackageErrorKind::AlreadyInstalled(Some(format!(
                "{} is already installed in your system.",
                meta_dir.meta.name
            )))
            .throw());
        }

        transaction_op(db, Transaction::Begin)?;

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

        try_bind_val!(sql, 1, &*meta_dir.meta.name);
        try_bind_val!(sql, 2, &*meta_dir.meta.description);
        try_bind_val!(sql, 3, &*meta_dir.meta.maintainer);
        try_bind_val!(sql, 4, SQLITE_NULL); // TODO

        if let Some(homepage) = &meta_dir.meta.homepage {
            try_bind_val!(sql, 5, &**homepage);
        } else {
            try_bind_val!(sql, 5, SQLITE_NULL);
        }

        if let Some(repository) = &meta_dir.meta.repository {
            let repository_id = get_repository_id_by_repository(db, repository)?;

            if let Some(r_id) = repository_id {
                try_bind_val!(sql, 6, r_id);
            } else {
                sql.kill();
                transaction_op(db, Transaction::Rollback)?;
                return Err(PackageErrorKind::UnrecognizedRepository(Some(format!(
                    "Repository '{}' is not defined in your system. See 'TODO'",
                    repository
                )))
                .throw());
            }
        } else {
            try_bind_val!(sql, 6, SQLITE_NULL);
        }

        try_bind_val!(sql, 7, 1_i32); // TODO
        try_bind_val!(sql, 8, meta_dir.meta.installed_size as i64);

        if let Some(license) = &meta_dir.meta.license {
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
            transaction_op(db, Transaction::Rollback)?;

            return Err(PackageErrorKind::InstallationFailed(Some(simple_e_fmt!(
                "Installing package \"{}\" is failed.",
                meta_dir.meta.name
            )))
            .throw());
        }

        let pkg_id = super::get_last_insert_row_id(db)?;

        sql.kill();

        match insert_pkg_tags(db, pkg_id, meta_dir.meta.tags.clone()) {
            Ok(_) => (),
            Err(err) => {
                transaction_op(db, Transaction::Rollback)?;
                return Err(err.into());
            }
        };

        match insert_files(db, pkg_id, &meta_dir.files) {
            Ok(_) => Ok(()),
            Err(err) => {
                transaction_op(db, Transaction::Rollback)?;
                Err(err)
            }
        }
    }

    fn from_db<'lpkg>(db: &Database, name: &str) -> Result<LodPkg<'lpkg>, PackageError> {
        let statement = String::from("SELECT * FROM packages WHERE name = ?;");
        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, 1, name);
        try_execute_prepared!(
            sql,
            Some(simple_e_fmt!("Error SELECT query on \"packages\" table."))
        );
        let id = sql.get_data::<i64>(0).unwrap_or(0);

        if id == 0 {
            sql.kill();
            return Err(PackageErrorKind::DoesNotExists(Some(format!(
                "{} is doesn't exists in your system.",
                name
            )))
            .throw());
        }

        let version = VersionStruct {
            major: sql.get_data(10).unwrap(),
            minor: sql.get_data(11).unwrap(),
            patch: sql.get_data(12).unwrap(),
            tag: sql.get_data(13).unwrap(),
            readable_format: sql.get_data(14).unwrap(),
        };

        let mut meta = Meta {
            name: sql.get_data(1).unwrap(),
            description: sql.get_data(2).unwrap(),
            maintainer: sql.get_data(3).unwrap(),
            source_pkg: None,
            repository: None,
            homepage: sql.get_data(5).unwrap(),
            arch: String::new(),
            kind: String::new(),
            installed_size: sql.get_data(8).unwrap(),
            tags: Vec::new(),
            version: version.clone(),
            license: sql.get_data(9).unwrap(),
            dependencies: Vec::new(),
            suggestions: Vec::new(),
        };

        let kind_id = sql.get_data::<i64>(7).unwrap();
        sql.kill();

        let kind_statement = String::from("SELECT kind FROM package_kinds WHERE id = ?;");
        let mut sql = db.prepare(kind_statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, 1, kind_id);

        while let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
            meta.kind = sql.get_data(0)?;
        }

        sql.kill();

        let tags_statement = String::from("SELECT tag FROM package_tags WHERE package_id = ?;");
        let mut sql = db.prepare(tags_statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, 1, id);

        while let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
            meta.tags.push(sql.get_data(0)?);
        }

        sql.kill();

        let files_statement = String::from("SELECT * FROM files WHERE package_id = ?;");
        let mut sql = db.prepare(files_statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, 1, id);

        let mut files: Vec<FileStruct> = Vec::new();

        while let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
            let file = FileStruct {
                path: sql.get_data(2).unwrap(),
                checksum_algorithm: get_checksum_algorithm_by_id(db, sql.get_data(4).unwrap())
                    .unwrap(),
                checksum: sql.get_data(3).unwrap(),
            };

            files.push(file);
        }
        sql.kill();

        let files = Files(files);

        let meta_dir = MetaDir {
            path: String::new(),
            meta,
            files,
        };

        Ok(LodPkg {
            path: None,
            meta_dir: Some(meta_dir),
            system: None,
            version,
        })
    }

    fn delete<'lpkg>(_db: &Database, _name: &str) -> Result<(), PackageError> {
        todo!()
    }
}

fn insert_files(db: &Database, pkg_id: i64, files: &Files) -> Result<(), PackageError> {
    let files = &files.0;

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

        try_execute_prepared!(
            sql,
            Some(simple_e_fmt!("Could not insert to \"files\" table."))
        );

        sql.kill();
    }

    Ok(())
}

pub fn is_package_exists(db: &Database, name: &str) -> Result<bool, SqlError> {
    let statement = String::from("SELECT EXISTS(SELECT 1 FROM packages WHERE name = ?);");

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, 1, name);
    try_execute_prepared!(
        sql,
        Some(simple_e_fmt!("Error SELECT query on \"packages\" table."))
    );

    let result = sql.get_data::<i64>(0).unwrap_or(0);
    sql.kill();

    Ok(result == 1)
}

pub fn get_repository_id_by_repository(
    db: &Database,
    repository: &str,
) -> Result<Option<i64>, SqlError> {
    let statement = String::from("SELECT id FROM repositories WHERE repository = ?;");

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, 1, repository);
    try_execute_prepared!(
        sql,
        Some(simple_e_fmt!(
            "Error SELECT query on \"repositories\" table."
        ))
    );

    let result = sql.get_data::<i64>(0).unwrap();
    sql.kill();

    Ok(Some(result))
}

pub fn get_checksum_algorithm_by_id(db: &Database, id: u8) -> Result<String, SqlError> {
    let statement = String::from("SELECT kind FROM checksum_kinds WHERE id = ?;");

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, 1, id);
    try_execute_prepared!(
        sql,
        Some(simple_e_fmt!(
            "Error SELECT query on \"checksum_kinds\" table."
        ))
    );

    let result = sql.get_data::<String>(0).unwrap();
    sql.kill();

    Ok(result)
}

pub fn get_checksum_algorithm_id_by_kind(
    db: &Database,
    algorithm: &str,
) -> Result<Option<i64>, SqlError> {
    let statement = String::from("SELECT id FROM checksum_kinds WHERE kind = ?;");

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, 1, algorithm);
    try_execute_prepared!(
        sql,
        Some(simple_e_fmt!(
            "Error SELECT query on \"checksum_kinds\" table."
        ))
    );

    let result = sql.get_data::<i64>(0).unwrap();
    sql.kill();

    Ok(Some(result))
}

/// This is a non-transactional insert operation. (created for `LodPkg::get_by_name` which
/// already has an opened transaction.)
/// To make it transactional, open&close the transaction from caller stack.
pub fn insert_pkg_tags(
    db: &Database,
    pkg_id: i64,
    tags: Vec<String>,
) -> Result<SqlitePrimaryResult, SqlError> {
    for tag in tags {
        let statement = String::from(
            "
                INSERT INTO package_tags
                    (tag, package_id)
                VALUES
                    (?, ?);",
        );

        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, 1, &*tag);
        try_bind_val!(sql, 2, pkg_id);
        try_execute_prepared!(
            sql,
            Some(simple_e_fmt!("Error on inserting package tag \"{}\".", tag))
        );
        sql.kill();
    }

    Ok(SqlitePrimaryResult::Ok)
}

pub fn insert_pkg_kinds(
    db: &Database,
    kinds: Vec<String>,
) -> Result<SqlitePrimaryResult, SqlError> {
    transaction_op(db, Transaction::Begin)?;

    for kind in kinds {
        let statement = String::from(
            "
                INSERT INTO package_kinds
                    (kind)
                VALUES
                    (?);",
        );

        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, 1, &*kind);
        try_execute_prepared!(
            sql,
            Some(simple_e_fmt!(
                "Error on inserting package kind \"{}\".",
                kind
            ))
        );
        sql.kill();
    }

    transaction_op(db, Transaction::Commit)
}

pub fn delete_pkg_kinds(
    db: &Database,
    kinds: Vec<String>,
) -> Result<SqlitePrimaryResult, SqlError> {
    transaction_op(db, Transaction::Begin)?;

    for kind in kinds {
        let statement = String::from(
            "
                DELETE FROM package_kinds
                WHERE
                    kind = ?;",
        );

        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, 1, &*kind);
        try_execute_prepared!(
            sql,
            Some(simple_e_fmt!(
                "Error on deleting package kind \"{}\".",
                kind
            ))
        );
        sql.kill();
    }

    transaction_op(db, Transaction::Commit)
}
