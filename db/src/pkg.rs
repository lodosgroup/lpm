use crate::sql_builder::delete::*;
use crate::sql_builder::insert::*;
use crate::sql_builder::select::*;
use crate::{enable_foreign_keys, transaction_op, Transaction};
use common::from_preprocessor;
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

        from_preprocessor!(NAME_COL_PRE_ID, 1);
        from_preprocessor!(DESCRIPTION_COL_PRE_ID, 2);
        from_preprocessor!(MAINTAINER_COL_PRE_ID, 3);
        from_preprocessor!(HOMEPAGE_COL_PRE_ID, 4);
        from_preprocessor!(DEPENDED_PACKAGE_ID_COL_PRE_ID, 5);
        from_preprocessor!(PACKAGE_KIND_ID_COL_PRE_ID, 6);
        from_preprocessor!(INSTALLED_SIZE_COL_PRE_ID, 7);
        from_preprocessor!(LICENSE_COL_PRE_ID, 8);
        from_preprocessor!(V_MAJOR_COL_PRE_ID, 9);
        from_preprocessor!(V_MINOR_COL_PRE_ID, 10);
        from_preprocessor!(V_PATCH_COL_PRE_ID, 11);
        from_preprocessor!(V_TAG_COL_PRE_ID, 12);
        from_preprocessor!(V_READABLE_COL_PRE_ID, 13);

        let package_columns = vec![
            Column::new(String::from("name"), NAME_COL_PRE_ID!()),
            Column::new(String::from("description"), DESCRIPTION_COL_PRE_ID!()),
            Column::new(String::from("maintainer"), MAINTAINER_COL_PRE_ID!()),
            Column::new(String::from("homepage"), HOMEPAGE_COL_PRE_ID!()),
            Column::new(
                String::from("depended_package_id"),
                DEPENDED_PACKAGE_ID_COL_PRE_ID!(),
            ),
            Column::new(
                String::from("package_kind_id"),
                PACKAGE_KIND_ID_COL_PRE_ID!(),
            ),
            Column::new(String::from("installed_size"), INSTALLED_SIZE_COL_PRE_ID!()),
            Column::new(String::from("license"), LICENSE_COL_PRE_ID!()),
            Column::new(String::from("v_major"), V_MAJOR_COL_PRE_ID!()),
            Column::new(String::from("v_minor"), V_MINOR_COL_PRE_ID!()),
            Column::new(String::from("v_patch"), V_PATCH_COL_PRE_ID!()),
            Column::new(String::from("v_tag"), V_TAG_COL_PRE_ID!()),
            Column::new(String::from("v_readable"), V_READABLE_COL_PRE_ID!()),
        ];

        let statement = Insert::new(Some(package_columns), String::from("packages")).to_string();

        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

        try_bind_val!(sql, NAME_COL_PRE_ID!(), &*meta_dir.meta.name);
        try_bind_val!(sql, DESCRIPTION_COL_PRE_ID!(), &*meta_dir.meta.description);
        try_bind_val!(sql, MAINTAINER_COL_PRE_ID!(), &*meta_dir.meta.maintainer);

        if let Some(homepage) = &meta_dir.meta.homepage {
            try_bind_val!(sql, HOMEPAGE_COL_PRE_ID!(), &**homepage);
        } else {
            try_bind_val!(sql, HOMEPAGE_COL_PRE_ID!(), SQLITE_NULL);
        }

        // TODO
        // will be used for sub-packages
        try_bind_val!(sql, DEPENDED_PACKAGE_ID_COL_PRE_ID!(), SQLITE_NULL);

        let kind_id = get_id_by_single_col_condition(
            db,
            String::from("package_kinds"),
            String::from("kind"),
            &meta_dir.meta.kind,
        )?
        .ok_or_else(|| {
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
        try_bind_val!(sql, PACKAGE_KIND_ID_COL_PRE_ID!(), kind_id);

        try_bind_val!(
            sql,
            INSTALLED_SIZE_COL_PRE_ID!(),
            meta_dir.meta.installed_size as i64
        );

        if let Some(license) = &meta_dir.meta.license {
            try_bind_val!(sql, LICENSE_COL_PRE_ID!(), &**license);
        } else {
            try_bind_val!(sql, LICENSE_COL_PRE_ID!(), SQLITE_NULL);
        }

        try_bind_val!(sql, V_MAJOR_COL_PRE_ID!(), meta_dir.meta.version.major);
        try_bind_val!(sql, V_MINOR_COL_PRE_ID!(), meta_dir.meta.version.minor);
        try_bind_val!(sql, V_PATCH_COL_PRE_ID!(), meta_dir.meta.version.patch);

        if let Some(vtag) = &meta_dir.meta.version.tag {
            try_bind_val!(sql, V_TAG_COL_PRE_ID!(), &**vtag);
        } else {
            try_bind_val!(sql, V_TAG_COL_PRE_ID!(), SQLITE_NULL);
        }

        try_bind_val!(
            sql,
            V_READABLE_COL_PRE_ID!(),
            &*meta_dir.meta.version.readable_format
        );

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
        from_preprocessor!(NAME_COL_PRE_ID, 1);
        let statement = Select::new(None, String::from("packages"))
            .where_condition(Where::Equal(NAME_COL_PRE_ID!(), String::from("name")))
            .to_string();
        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, NAME_COL_PRE_ID!(), name);
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

        from_preprocessor!(ID_COL_PRE_ID, 1);
        let kind_statement = Select::new(
            Some(vec![String::from("kind")]),
            String::from("package_kinds"),
        )
        .where_condition(Where::Equal(ID_COL_PRE_ID!(), String::from("id")))
        .to_string();
        let mut sql = db.prepare(kind_statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, ID_COL_PRE_ID!(), kind_id);

        while let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
            meta.kind = sql.get_data(0)?;
        }

        sql.kill();

        from_preprocessor!(PACKAGE_ID_COL_PRE_ID, 1);
        let tags_statement = Select::new(
            Some(vec![String::from("tag")]),
            String::from("package_tags"),
        )
        .where_condition(Where::Equal(
            PACKAGE_ID_COL_PRE_ID!(),
            String::from("package_id"),
        ))
        .to_string();
        let mut sql = db.prepare(tags_statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, PACKAGE_ID_COL_PRE_ID!(), id);

        while let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
            meta.tags.push(sql.get_data(0)?);
        }

        sql.kill();

        let files_statement = Select::new(None, String::from("files"))
            .where_condition(Where::Equal(
                PACKAGE_ID_COL_PRE_ID!(),
                String::from("package_id"),
            ))
            .to_string();
        let mut sql = db.prepare(files_statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, PACKAGE_ID_COL_PRE_ID!(), id);

        let mut files: Vec<FileStruct> = Vec::new();

        while let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
            let file = FileStruct {
                path: sql.get_data(2)?,
                checksum_algorithm: get_string_value_by_id(
                    db,
                    String::from("checksum_kinds"),
                    String::from("kind"),
                    sql.get_data(4)?,
                )?,
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
        enable_foreign_keys(db)?;
        from_preprocessor!(NAME_COL_PRE_ID, 1);
        let statement = Delete::new(String::from("packages"))
            .where_condition(Where::Equal(NAME_COL_PRE_ID!(), String::from("name")))
            .to_string();

        let pkg_name = &self
            .meta_dir
            .as_ref()
            .ok_or_else(|| PackageErrorKind::MetaDirCouldNotLoad.to_lpm_err())?
            .meta
            .name;

        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, NAME_COL_PRE_ID!(), pkg_name.clone());
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
        let checksum_id = get_id_by_single_col_condition(
            db,
            String::from("checksum_kinds"),
            String::from("kind"),
            &file.checksum_algorithm,
        )?;
        if checksum_id.is_none() {
            return Err(PackageErrorKind::UnsupportedChecksumAlgorithm(
                file.checksum_algorithm.clone(),
            )
            .to_lpm_err());
        }

        let file_path = Path::new(&file.path);

        from_preprocessor!(NAME_COL_PRE_ID, 1);
        from_preprocessor!(ABSOLUTE_PATH_COL_PRE_ID, 2);
        from_preprocessor!(CHECKSUM_COL_PRE_ID, 3);
        from_preprocessor!(CHECKSUM_KIND_ID_COL_PRE_ID, 4);
        from_preprocessor!(PACKAGE_ID_COL_PRE_ID, 5);

        let file_columns = vec![
            Column::new(String::from("name"), NAME_COL_PRE_ID!()),
            Column::new(String::from("absolute_path"), ABSOLUTE_PATH_COL_PRE_ID!()),
            Column::new(String::from("checksum"), CHECKSUM_COL_PRE_ID!()),
            Column::new(
                String::from("checksum_kind_id"),
                CHECKSUM_KIND_ID_COL_PRE_ID!(),
            ),
            Column::new(String::from("package_id"), PACKAGE_ID_COL_PRE_ID!()),
        ];
        let statement = Insert::new(Some(file_columns), String::from("files")).to_string();

        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

        try_bind_val!(
            sql,
            NAME_COL_PRE_ID!(),
            file_path.file_name().unwrap().to_str().unwrap()
        );
        try_bind_val!(sql, ABSOLUTE_PATH_COL_PRE_ID!(), format!("/{}", &file.path));
        try_bind_val!(sql, CHECKSUM_COL_PRE_ID!(), &*file.checksum);
        try_bind_val!(sql, CHECKSUM_KIND_ID_COL_PRE_ID!(), checksum_id.unwrap());
        try_bind_val!(sql, PACKAGE_ID_COL_PRE_ID!(), pkg_id);

        try_execute_prepared!(sql, simple_e_fmt!("Could not insert to \"files\" table."));

        sql.kill();
    }

    Ok(())
}

fn is_package_exists(db: &Database, name: &str) -> Result<bool, LpmError<SqlError>> {
    from_preprocessor!(NAME_COL_PRE_ID, 1);
    let exists_statement = Select::new(None, String::from("packages"))
        .where_condition(Where::Equal(NAME_COL_PRE_ID!(), String::from("name")))
        .exists()
        .to_string();

    let mut sql = db.prepare(exists_statement.clone(), super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, NAME_COL_PRE_ID!(), name);

    try_execute_prepared!(
        sql,
        simple_e_fmt!("Select exists query failed. SQL:\n {}", exists_statement)
    );

    let result = sql.get_data::<i64>(0).unwrap_or(0);
    sql.kill();

    Ok(result == 1)
}

fn get_id_by_single_col_condition(
    db: &Database,
    table: String,
    column: String,
    value: &str,
) -> Result<Option<i64>, LpmError<SqlError>> {
    from_preprocessor!(COL_PRE_ID, 1);
    let get_id_statement = Select::new(Some(vec![String::from("id")]), table)
        .where_condition(Where::Equal(COL_PRE_ID!(), column))
        .to_string();

    let mut sql = db.prepare(get_id_statement.clone(), super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, COL_PRE_ID!(), value);

    try_execute_prepared!(
        sql,
        simple_e_fmt!("Select id query failed. SQL:\n {}", get_id_statement)
    );

    let result = sql.get_data::<Option<i64>>(0)?;
    sql.kill();

    Ok(result)
}

fn get_string_value_by_id(
    db: &Database,
    table: String,
    column: String,
    id: u32,
) -> Result<String, LpmError<SqlError>> {
    from_preprocessor!(COL_PRE_ID, 1);
    let statement = Select::new(Some(vec![column]), table)
        .where_condition(Where::Equal(COL_PRE_ID!(), String::from("id")))
        .to_string();

    let mut sql = db.prepare(statement.clone(), super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, COL_PRE_ID!(), id);
    try_execute_prepared!(
        sql,
        simple_e_fmt!("Select query failed. SQL:\n {}", statement)
    );

    let result = sql.get_data::<String>(0)?;
    sql.kill();

    Ok(result)
}

/// Batch insert of package tags on sqlite
fn insert_pkg_tags(
    db: &Database,
    pkg_id: i64,
    tags: Vec<String>,
) -> Result<PreparedStatementStatus, LpmError<SqlError>> {
    from_preprocessor!(TAG_COL_PRE_ID, 1);
    from_preprocessor!(PACKAGE_ID_COL_PRE_ID, 255);
    let package_tag_columns = vec![
        Column::new(String::from("tag"), TAG_COL_PRE_ID!()),
        Column::new(String::from("package_id"), PACKAGE_ID_COL_PRE_ID!()),
    ];

    let mut sql_builder = Insert::new(Some(package_tag_columns), String::from("package_tags"));

    for (index, _) in tags.iter().enumerate() {
        let index = index + 1;
        if index == PACKAGE_ID_COL_PRE_ID!() || index == TAG_COL_PRE_ID!() {
            continue;
        }

        sql_builder = sql_builder.insert_another_row(vec![index as u8, PACKAGE_ID_COL_PRE_ID!()]);
    }

    let statement = sql_builder.to_string();
    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    for (index, tag) in tags.iter().enumerate() {
        let index = index + 1;
        try_bind_val!(sql, index, &**tag);
        try_bind_val!(sql, PACKAGE_ID_COL_PRE_ID!(), pkg_id);
    }

    let tags = tags.join(", ");
    debug!("Inserting tags {}", tags);

    let status = try_execute_prepared!(
        sql,
        simple_e_fmt!("Error on inserting package tags '{}'.", tags)
    );

    sql.kill();

    Ok(status)
}

/// Batch insert of package kinds on sqlite
pub fn insert_pkg_kinds(
    db: &Database,
    kinds: Vec<String>,
) -> Result<PreparedStatementStatus, LpmError<SqlError>> {
    from_preprocessor!(KIND_COL_PRE_ID, 1);
    let package_kind_columns = vec![Column::new(String::from("kind"), KIND_COL_PRE_ID!())];
    let mut sql_builder = Insert::new(Some(package_kind_columns), String::from("package_kinds"));

    for (index, _) in kinds.iter().enumerate() {
        let index = index + 1;
        if index == KIND_COL_PRE_ID!() {
            continue;
        }

        sql_builder = sql_builder.insert_another_row(vec![index as u8]);
    }

    let statement = sql_builder.to_string();

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
    for (index, kind) in kinds.iter().enumerate() {
        let index = index + 1;
        try_bind_val!(sql, index, &**kind);
    }

    let kinds = kinds.join(", ");
    debug!("Inserting kinds {}", kinds);
    let status = try_execute_prepared!(
        sql,
        simple_e_fmt!("Error on inserting package kinds '{}'.", kinds)
    );

    sql.kill();

    Ok(status)
}

/// Batch delete of package kinds on sqlite
pub fn delete_pkg_kinds(
    db: &Database,
    kinds: Vec<String>,
) -> Result<PreparedStatementStatus, LpmError<SqlError>> {
    enable_foreign_keys(db)?;

    let mut pre_ids = vec![];
    for (index, _) in kinds.iter().enumerate() {
        pre_ids.push(index as u8 + 1_u8);
    }

    let statement = Delete::new(String::from("package_kinds"))
        .where_condition(Where::In(pre_ids, String::from("kind")))
        .to_string();

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
    for (index, kind) in kinds.iter().enumerate() {
        try_bind_val!(sql, index + 1, &**kind);
    }

    let kinds = kinds.join(", ");
    debug!("Deleting kinds {}", kinds);
    let status = try_execute_prepared!(
        sql,
        simple_e_fmt!("Error on deleting package kinds '{}'. One of the kinds you are trying to delete might be in use from installed packages.", kinds)
    );

    sql.kill();

    Ok(status)
}
