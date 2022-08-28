use crate::{enable_foreign_keys, transaction_op, Transaction};
use common::{
    meta::{FileStruct, Meta},
    pkg::{LodPkg, MetaDir},
    version::VersionStruct,
    Files,
};
use ehandle::{
    db::SqlError,
    lpm::LpmError,
    pkg::{PackageError, PackageErrorKind},
    simple_e_fmt, try_bind_val, try_execute_prepared, ErrorCommons,
};
use min_sqlite3_sys::prelude::*;
use std::path::Path;
use term::{debug, info};

pub trait LodPkgCoreDbOps {
    fn from_db<'lpkg>(db: &Database, name: &str) -> Result<LodPkg<'lpkg>, LpmError<PackageError>>;
    fn insert_to_db(&self, db: &Database) -> Result<(), LpmError<PackageError>>;
    fn delete_from_db(&self, db: &Database) -> Result<(), LpmError<PackageError>>;
}

impl<'a> LodPkgCoreDbOps for LodPkg<'a> {
    fn insert_to_db(&self, db: &Database) -> Result<(), LpmError<PackageError>> {
        enable_foreign_keys(db)?;

        let meta_dir = self
            .meta_dir
            .as_ref()
            .ok_or_else(|| PackageErrorKind::MetaDirCouldNotLoad.to_lpm_err())?;

        if is_package_exists(db, &meta_dir.meta.name)? {
            return Err(
                PackageErrorKind::AlreadyInstalled(meta_dir.meta.name.clone()).to_lpm_err(),
            );
        }

        transaction_op(db, Transaction::Begin)?;

        let statement = String::from(
            "
                INSERT INTO packages
                    (name, description, maintainer, homepage, 
                    depended_package_id, package_kind_id,
                    installed_size, license, v_major, v_minor, v_patch,
                    v_tag, v_readable)
                VALUES
                    (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        );

        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

        try_bind_val!(sql, 1, &*meta_dir.meta.name);
        try_bind_val!(sql, 2, &*meta_dir.meta.description);
        try_bind_val!(sql, 3, &*meta_dir.meta.maintainer);

        if let Some(homepage) = &meta_dir.meta.homepage {
            try_bind_val!(sql, 4, &**homepage);
        } else {
            try_bind_val!(sql, 4, SQLITE_NULL);
        }

        // TODO
        // will be used for sub-packages
        try_bind_val!(sql, 5, SQLITE_NULL);

        let kind_id = get_kind_id_by_kind_name(db, &meta_dir.meta.kind)?.ok_or_else(|| {
            PackageErrorKind::PackageKindNotFound(meta_dir.meta.kind.clone()).to_lpm_err()
        });
        let kind_id = match kind_id {
            Ok(id) => id,
            Err(e) => {
                sql.kill();
                transaction_op(db, Transaction::Rollback)?;
                return Err(e);
            }
        };
        try_bind_val!(sql, 6, kind_id);

        try_bind_val!(sql, 7, meta_dir.meta.installed_size as i64);

        if let Some(license) = &meta_dir.meta.license {
            try_bind_val!(sql, 8, &**license);
        } else {
            try_bind_val!(sql, 8, SQLITE_NULL);
        }

        try_bind_val!(sql, 9, meta_dir.meta.version.major);
        try_bind_val!(sql, 10, meta_dir.meta.version.minor);
        try_bind_val!(sql, 11, meta_dir.meta.version.patch);

        if let Some(vtag) = &meta_dir.meta.version.tag {
            try_bind_val!(sql, 12, &**vtag);
        } else {
            try_bind_val!(sql, 12, SQLITE_NULL);
        }

        try_bind_val!(sql, 13, &*meta_dir.meta.version.readable_format);

        if PreparedStatementStatus::Done != sql.execute_prepared() {
            sql.kill();
            transaction_op(db, Transaction::Rollback)?;

            return Err(
                PackageErrorKind::InstallationFailed(meta_dir.meta.name.clone()).to_lpm_err(),
            );
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

    fn from_db<'lpkg>(db: &Database, name: &str) -> Result<LodPkg<'lpkg>, LpmError<PackageError>> {
        info!("Loading '{}' from database..", name);
        let statement = String::from("SELECT * FROM packages WHERE name = ?;");
        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, 1, name);
        try_execute_prepared!(
            sql,
            simple_e_fmt!("Error SELECT query on \"packages\" table.")
        );
        let id = sql.get_data::<i64>(0).unwrap_or(0);

        if id == 0 {
            sql.kill();
            return Err(PackageErrorKind::DoesNotExists(name.to_string()).to_lpm_err());
        }

        let version = VersionStruct {
            major: sql.get_data(10)?,
            minor: sql.get_data(11)?,
            patch: sql.get_data(12)?,
            tag: sql.get_data(13)?,
            readable_format: sql.get_data(14)?,
        };

        let mut meta = Meta {
            name: sql.get_data(1)?,
            description: sql.get_data(2)?,
            maintainer: sql.get_data(3)?,
            source_pkg: None,
            repository: None,
            homepage: sql.get_data(5)?,
            arch: String::new(),
            kind: String::new(),
            installed_size: sql.get_data(8)?,
            tags: Vec::new(),
            version: version.clone(),
            license: sql.get_data(9)?,
            dependencies: Vec::new(),
            suggestions: Vec::new(),
        };

        let kind_id = sql.get_data::<i64>(7)?;
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
                path: sql.get_data(2)?,
                checksum_algorithm: get_checksum_algorithm_by_id(db, sql.get_data(4)?)?,
                checksum: sql.get_data(3)?,
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

        info!("Package '{}' successfully loaded.", name);
        Ok(LodPkg {
            path: None,
            meta_dir: Some(meta_dir),
            system: None,
            version,
        })
    }

    fn delete_from_db<'lpkg>(&self, db: &Database) -> Result<(), LpmError<PackageError>> {
        let statement = String::from(
            "
                DELETE FROM packages
                WHERE
                    name = ?;",
        );

        let pkg_name = &self
            .meta_dir
            .as_ref()
            .ok_or_else(|| PackageErrorKind::MetaDirCouldNotLoad.to_lpm_err())?
            .meta
            .name;

        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, 1, pkg_name.clone());
        try_execute_prepared!(
            sql,
            simple_e_fmt!("Error on deleting package \"{}\".", pkg_name)
        );
        sql.kill();

        Ok(())
    }
}

fn insert_files(db: &Database, pkg_id: i64, files: &Files) -> Result<(), LpmError<PackageError>> {
    let files = &files.0;

    for file in files {
        let checksum_id = get_checksum_algorithm_id_by_kind(db, &file.checksum_algorithm)?;
        if checksum_id.is_none() {
            return Err(PackageErrorKind::UnsupportedChecksumAlgorithm(
                file.checksum_algorithm.clone(),
            )
            .to_lpm_err());
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

        try_execute_prepared!(sql, simple_e_fmt!("Could not insert to \"files\" table."));

        sql.kill();
    }

    Ok(())
}

pub fn is_package_exists(db: &Database, name: &str) -> Result<bool, LpmError<SqlError>> {
    let statement = String::from("SELECT EXISTS(SELECT 1 FROM packages WHERE name = ?);");

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, 1, name);
    try_execute_prepared!(
        sql,
        simple_e_fmt!("Error SELECT query on \"packages\" table.")
    );

    let result = sql.get_data::<i64>(0).unwrap_or(0);
    sql.kill();

    Ok(result == 1)
}

pub fn get_repository_id_by_repository(
    db: &Database,
    repository: &str,
) -> Result<Option<i64>, LpmError<SqlError>> {
    let statement = String::from("SELECT id FROM repositories WHERE repository = ?;");

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, 1, repository);
    try_execute_prepared!(
        sql,
        simple_e_fmt!("Error SELECT query on \"repositories\" table.")
    );

    let result = sql.get_data::<Option<i64>>(0)?;
    sql.kill();

    Ok(result)
}

pub fn get_checksum_algorithm_by_id(db: &Database, id: u8) -> Result<String, LpmError<SqlError>> {
    let statement = String::from("SELECT kind FROM checksum_kinds WHERE id = ?;");

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, 1, id);
    try_execute_prepared!(
        sql,
        simple_e_fmt!("Error SELECT query on \"checksum_kinds\" table.")
    );

    let result = sql.get_data::<String>(0)?;
    sql.kill();

    Ok(result)
}

pub fn get_kind_id_by_kind_name(
    db: &Database,
    kind: &str,
) -> Result<Option<i64>, LpmError<SqlError>> {
    let statement = String::from("SELECT id FROM package_kinds WHERE kind = ?;");

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, 1, kind);
    try_execute_prepared!(
        sql,
        simple_e_fmt!("Error SELECT query on \"package_kinds\" table.")
    );

    let result = sql.get_data::<i64>(0)?;
    sql.kill();

    if result == 0 {
        return Ok(None);
    }

    Ok(Some(result))
}

pub fn get_checksum_algorithm_id_by_kind(
    db: &Database,
    algorithm: &str,
) -> Result<Option<i64>, LpmError<SqlError>> {
    let statement = String::from("SELECT id FROM checksum_kinds WHERE kind = ?;");

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, 1, algorithm);
    try_execute_prepared!(
        sql,
        simple_e_fmt!("Error SELECT query on \"checksum_kinds\" table.")
    );

    let result = sql.get_data::<i64>(0)?;
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
) -> Result<SqlitePrimaryResult, LpmError<SqlError>> {
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
            simple_e_fmt!("Error on inserting package tag \"{}\".", tag)
        );
        sql.kill();
    }

    Ok(SqlitePrimaryResult::Ok)
}

pub fn insert_pkg_kinds(
    db: &Database,
    kinds: Vec<String>,
) -> Result<SqlitePrimaryResult, LpmError<SqlError>> {
    transaction_op(db, Transaction::Begin)?;

    for kind in kinds {
        debug!("Inserting kind {}", kind);
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
            simple_e_fmt!("Error on inserting package kind '{}'.", kind)
        );
        sql.kill();
    }

    transaction_op(db, Transaction::Commit)
}

pub fn delete_pkg_kinds(
    db: &Database,
    kinds: Vec<String>,
) -> Result<SqlitePrimaryResult, LpmError<SqlError>> {
    transaction_op(db, Transaction::Begin)?;

    for kind in kinds {
        debug!("Deleting kind {}", kind);
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
            simple_e_fmt!("Error on deleting package kind \"{}\".", kind)
        );
        sql.kill();
    }

    transaction_op(db, Transaction::Commit)
}
