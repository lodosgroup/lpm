use crate::{enable_foreign_keys, transaction_op, Transaction};
use common::pkg::PkgDataFromDb;
use common::pkg::PkgDataFromFs;
use common::{
    meta::{FileStruct, Meta},
    pkg::MetaDir,
    version::VersionStruct,
    Files,
};
use ehandle::{
    db::SqlError,
    lpm::LpmError,
    pkg::{PackageError, PackageErrorKind},
    simple_e_fmt, try_bind_val, try_execute_prepared, ErrorCommons,
};
use logger::{debug, info};
use min_sqlite3_sys::prelude::*;
use sql_builder::delete::*;
use sql_builder::insert::*;
use sql_builder::select::*;
use sql_builder::update::Update;
use sql_builder::Column;
use std::path::Path;

pub trait DbOpsForInstalledPkg {
    fn load(db: &Database, name: &str) -> Result<Self, LpmError<PackageError>>
    where
        Self: Sized;
    fn delete_from_db(&self, db: &Database) -> Result<(), LpmError<PackageError>>;
}

pub trait DbOpsForBuildFile {
    fn insert_to_db(&self, db: &Database) -> Result<(), LpmError<PackageError>>;
    fn update_existing_pkg(&self, db: &Database, pkg_id: i64)
        -> Result<(), LpmError<PackageError>>;
}

impl DbOpsForBuildFile for PkgDataFromFs {
    fn insert_to_db(&self, db: &Database) -> Result<(), LpmError<PackageError>> {
        enable_foreign_keys(db)?;

        if is_package_exists(db, &self.meta_dir.meta.name)? {
            return Err(
                PackageErrorKind::AlreadyInstalled(self.meta_dir.meta.name.clone()).to_lpm_err(),
            );
        }

        transaction_op(db, Transaction::Begin)?;

        const NAME_COL_PRE_ID: usize = 1;
        const DESCRIPTION_COL_PRE_ID: usize = 2;
        const MAINTAINER_COL_PRE_ID: usize = 3;
        const HOMEPAGE_COL_PRE_ID: usize = 4;
        const SRC_PKG_ID_COL_PRE_ID: usize = 5;
        const PACKAGE_KIND_ID_COL_PRE_ID: usize = 6;
        const INSTALLED_SIZE_COL_PRE_ID: usize = 7;
        const LICENSE_COL_PRE_ID: usize = 8;
        const V_MAJOR_COL_PRE_ID: usize = 9;
        const V_MINOR_COL_PRE_ID: usize = 10;
        const V_PATCH_COL_PRE_ID: usize = 11;
        const V_TAG_COL_PRE_ID: usize = 12;
        const V_READABLE_COL_PRE_ID: usize = 13;

        let package_columns = vec![
            Column::new(String::from("name"), NAME_COL_PRE_ID),
            Column::new(String::from("description"), DESCRIPTION_COL_PRE_ID),
            Column::new(String::from("maintainer"), MAINTAINER_COL_PRE_ID),
            Column::new(String::from("homepage"), HOMEPAGE_COL_PRE_ID),
            Column::new(String::from("src_pkg_package_id"), SRC_PKG_ID_COL_PRE_ID),
            Column::new(String::from("package_kind_id"), PACKAGE_KIND_ID_COL_PRE_ID),
            Column::new(String::from("installed_size"), INSTALLED_SIZE_COL_PRE_ID),
            Column::new(String::from("license"), LICENSE_COL_PRE_ID),
            Column::new(String::from("v_major"), V_MAJOR_COL_PRE_ID),
            Column::new(String::from("v_minor"), V_MINOR_COL_PRE_ID),
            Column::new(String::from("v_patch"), V_PATCH_COL_PRE_ID),
            Column::new(String::from("v_tag"), V_TAG_COL_PRE_ID),
            Column::new(String::from("v_readable"), V_READABLE_COL_PRE_ID),
        ];

        let statement = Insert::new(Some(package_columns), String::from("packages")).to_string();

        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

        try_bind_val!(sql, NAME_COL_PRE_ID, &*self.meta_dir.meta.name);
        try_bind_val!(
            sql,
            DESCRIPTION_COL_PRE_ID,
            &*self.meta_dir.meta.description
        );
        try_bind_val!(sql, MAINTAINER_COL_PRE_ID, &*self.meta_dir.meta.maintainer);

        if let Some(homepage) = &self.meta_dir.meta.homepage {
            try_bind_val!(sql, HOMEPAGE_COL_PRE_ID, &**homepage);
        } else {
            try_bind_val!(sql, HOMEPAGE_COL_PRE_ID, SQLITE_NULL);
        }

        // TODO
        // will be used for sub-packages
        try_bind_val!(sql, SRC_PKG_ID_COL_PRE_ID, SQLITE_NULL);

        let kind_id = get_id_by_single_col_condition(
            db,
            String::from("package_kinds"),
            String::from("kind"),
            &self.meta_dir.meta.kind,
        )?
        .ok_or_else(|| {
            PackageErrorKind::PackageKindNotFound(self.meta_dir.meta.kind.clone()).to_lpm_err()
        });
        let kind_id = match kind_id {
            Ok(id) => id,
            Err(e) => {
                sql.kill();
                transaction_op(db, Transaction::Rollback)?;
                return Err(e);
            }
        };
        try_bind_val!(sql, PACKAGE_KIND_ID_COL_PRE_ID, kind_id);

        try_bind_val!(
            sql,
            INSTALLED_SIZE_COL_PRE_ID,
            self.meta_dir.meta.installed_size
        );

        if let Some(license) = &self.meta_dir.meta.license {
            try_bind_val!(sql, LICENSE_COL_PRE_ID, &**license);
        } else {
            try_bind_val!(sql, LICENSE_COL_PRE_ID, SQLITE_NULL);
        }

        try_bind_val!(sql, V_MAJOR_COL_PRE_ID, self.meta_dir.meta.version.major);
        try_bind_val!(sql, V_MINOR_COL_PRE_ID, self.meta_dir.meta.version.minor);
        try_bind_val!(sql, V_PATCH_COL_PRE_ID, self.meta_dir.meta.version.patch);

        if let Some(vtag) = &self.meta_dir.meta.version.tag {
            try_bind_val!(sql, V_TAG_COL_PRE_ID, &**vtag);
        } else {
            try_bind_val!(sql, V_TAG_COL_PRE_ID, SQLITE_NULL);
        }

        try_bind_val!(
            sql,
            V_READABLE_COL_PRE_ID,
            &*self.meta_dir.meta.version.readable_format
        );

        if PreparedStatementStatus::Done != sql.execute_prepared() {
            sql.kill();
            transaction_op(db, Transaction::Rollback)?;

            return Err(
                PackageErrorKind::InstallationFailed(self.meta_dir.meta.name.clone()).to_lpm_err(),
            );
        }

        let pkg_id = super::get_last_insert_row_id(db)?;

        sql.kill();

        match insert_pkg_tags(db, pkg_id, self.meta_dir.meta.tags.clone()) {
            Ok(_) => (),
            Err(err) => {
                transaction_op(db, Transaction::Rollback)?;
                return Err(err.into());
            }
        };

        match insert_files(db, pkg_id, &self.meta_dir.files) {
            Ok(_) => Ok(()),
            Err(err) => {
                transaction_op(db, Transaction::Rollback)?;
                Err(err)
            }
        }
    }

    fn update_existing_pkg(
        &self,
        db: &Database,
        pkg_id: i64,
    ) -> Result<(), LpmError<PackageError>> {
        enable_foreign_keys(db)?;

        if !is_package_exists(db, &self.meta_dir.meta.name)? {
            return Err(
                PackageErrorKind::DoesNotExists(self.meta_dir.meta.name.clone()).to_lpm_err(),
            );
        }

        transaction_op(db, Transaction::Begin)?;

        const NAME_COL_PRE_ID: usize = 1;
        const DESCRIPTION_COL_PRE_ID: usize = 2;
        const MAINTAINER_COL_PRE_ID: usize = 3;
        const HOMEPAGE_COL_PRE_ID: usize = 4;
        const SRC_PKG_ID_COL_PRE_ID: usize = 5;
        const PACKAGE_KIND_ID_COL_PRE_ID: usize = 6;
        const INSTALLED_SIZE_COL_PRE_ID: usize = 7;
        const LICENSE_COL_PRE_ID: usize = 8;
        const V_MAJOR_COL_PRE_ID: usize = 9;
        const V_MINOR_COL_PRE_ID: usize = 10;
        const V_PATCH_COL_PRE_ID: usize = 11;
        const V_TAG_COL_PRE_ID: usize = 12;
        const V_READABLE_COL_PRE_ID: usize = 13;

        let update_fields = vec![
            Column::new(String::from("description"), DESCRIPTION_COL_PRE_ID),
            Column::new(String::from("maintainer"), MAINTAINER_COL_PRE_ID),
            Column::new(String::from("homepage"), HOMEPAGE_COL_PRE_ID),
            Column::new(String::from("src_pkg_package_id"), SRC_PKG_ID_COL_PRE_ID),
            Column::new(String::from("package_kind_id"), PACKAGE_KIND_ID_COL_PRE_ID),
            Column::new(String::from("installed_size"), INSTALLED_SIZE_COL_PRE_ID),
            Column::new(String::from("license"), LICENSE_COL_PRE_ID),
            Column::new(String::from("v_major"), V_MAJOR_COL_PRE_ID),
            Column::new(String::from("v_minor"), V_MINOR_COL_PRE_ID),
            Column::new(String::from("v_patch"), V_PATCH_COL_PRE_ID),
            Column::new(String::from("v_tag"), V_TAG_COL_PRE_ID),
            Column::new(String::from("v_readable"), V_READABLE_COL_PRE_ID),
        ];

        let statement = Update::new(update_fields, String::from("packages"))
            .where_condition(Where::Equal(NAME_COL_PRE_ID, String::from("name")))
            .to_string();

        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

        try_bind_val!(sql, NAME_COL_PRE_ID, &*self.meta_dir.meta.name);
        try_bind_val!(
            sql,
            DESCRIPTION_COL_PRE_ID,
            &*self.meta_dir.meta.description
        );
        try_bind_val!(sql, MAINTAINER_COL_PRE_ID, &*self.meta_dir.meta.maintainer);

        if let Some(homepage) = &self.meta_dir.meta.homepage {
            try_bind_val!(sql, HOMEPAGE_COL_PRE_ID, &**homepage);
        } else {
            try_bind_val!(sql, HOMEPAGE_COL_PRE_ID, SQLITE_NULL);
        }

        // TODO
        // will be used for sub-packages
        try_bind_val!(sql, SRC_PKG_ID_COL_PRE_ID, SQLITE_NULL);

        let kind_id = get_id_by_single_col_condition(
            db,
            String::from("package_kinds"),
            String::from("kind"),
            &self.meta_dir.meta.kind,
        )?
        .ok_or_else(|| {
            PackageErrorKind::PackageKindNotFound(self.meta_dir.meta.kind.clone()).to_lpm_err()
        });
        let kind_id = match kind_id {
            Ok(id) => id,
            Err(e) => {
                sql.kill();
                transaction_op(db, Transaction::Rollback)?;
                return Err(e);
            }
        };
        try_bind_val!(sql, PACKAGE_KIND_ID_COL_PRE_ID, kind_id);

        try_bind_val!(
            sql,
            INSTALLED_SIZE_COL_PRE_ID,
            self.meta_dir.meta.installed_size
        );

        if let Some(license) = &self.meta_dir.meta.license {
            try_bind_val!(sql, LICENSE_COL_PRE_ID, &**license);
        } else {
            try_bind_val!(sql, LICENSE_COL_PRE_ID, SQLITE_NULL);
        }

        try_bind_val!(sql, V_MAJOR_COL_PRE_ID, self.meta_dir.meta.version.major);
        try_bind_val!(sql, V_MINOR_COL_PRE_ID, self.meta_dir.meta.version.minor);
        try_bind_val!(sql, V_PATCH_COL_PRE_ID, self.meta_dir.meta.version.patch);

        if let Some(vtag) = &self.meta_dir.meta.version.tag {
            try_bind_val!(sql, V_TAG_COL_PRE_ID, &**vtag);
        } else {
            try_bind_val!(sql, V_TAG_COL_PRE_ID, SQLITE_NULL);
        }

        try_bind_val!(
            sql,
            V_READABLE_COL_PRE_ID,
            &*self.meta_dir.meta.version.readable_format
        );

        if PreparedStatementStatus::Done != sql.execute_prepared() {
            sql.kill();
            transaction_op(db, Transaction::Rollback)?;

            return Err(
                PackageErrorKind::InstallationFailed(self.meta_dir.meta.name.clone()).to_lpm_err(),
            );
        }

        sql.kill();

        match delete_pkg_tags(db, pkg_id) {
            Ok(_) => (),
            Err(err) => {
                transaction_op(db, Transaction::Rollback)?;
                return Err(err.into());
            }
        };

        match insert_pkg_tags(db, pkg_id, self.meta_dir.meta.tags.clone()) {
            Ok(_) => (),
            Err(err) => {
                transaction_op(db, Transaction::Rollback)?;
                return Err(err.into());
            }
        };

        match delete_pkg_files(db, pkg_id) {
            Ok(_) => (),
            Err(err) => {
                transaction_op(db, Transaction::Rollback)?;
                return Err(err.into());
            }
        };

        match insert_files(db, pkg_id, &self.meta_dir.files) {
            Ok(_) => Ok(()),
            Err(err) => {
                transaction_op(db, Transaction::Rollback)?;
                Err(err)
            }
        }
    }
}

impl DbOpsForInstalledPkg for PkgDataFromDb {
    fn load(db: &Database, name: &str) -> Result<Self, LpmError<PackageError>> {
        info!("Loading '{}' from database..", name);

        const PKG_ID_COL_PRE_ID: usize = 0;
        const NAME_COL_PRE_ID: usize = 1;
        const DESCRIPTION_COL_PRE_ID: usize = 2;
        const MAINTAINER_COL_PRE_ID: usize = 3;
        const HOMEPAGE_COL_PRE_ID: usize = 4;
        // const SRC_PKG_ID_COL_PRE_ID: usize = 5;
        const PACKAGE_KIND_ID_COL_PRE_ID: usize = 6;
        const INSTALLED_SIZE_COL_PRE_ID: usize = 7;
        const LICENSE_COL_PRE_ID: usize = 8;
        const V_MAJOR_COL_PRE_ID: usize = 9;
        const V_MINOR_COL_PRE_ID: usize = 10;
        const V_PATCH_COL_PRE_ID: usize = 11;
        const V_TAG_COL_PRE_ID: usize = 12;
        const V_READABLE_COL_PRE_ID: usize = 13;

        let statement = Select::new(None, String::from("packages"))
            .where_condition(Where::Equal(NAME_COL_PRE_ID, String::from("name")))
            .to_string();
        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, NAME_COL_PRE_ID, name);
        try_execute_prepared!(
            sql,
            simple_e_fmt!("Error SELECT query on 'packages' table.")
        );
        let id: i64 = sql.get_data(PKG_ID_COL_PRE_ID).unwrap_or(0);

        if id == 0 {
            sql.kill();
            return Err(PackageErrorKind::DoesNotExists(name.to_string()).to_lpm_err());
        }

        let version = VersionStruct {
            major: sql.get_data(V_MAJOR_COL_PRE_ID)?,
            minor: sql.get_data(V_MINOR_COL_PRE_ID)?,
            patch: sql.get_data(V_PATCH_COL_PRE_ID)?,
            tag: sql.get_data(V_TAG_COL_PRE_ID)?,
            readable_format: sql.get_data(V_READABLE_COL_PRE_ID)?,
        };

        let mut meta = Meta {
            name: sql.get_data(NAME_COL_PRE_ID)?,
            description: sql.get_data(DESCRIPTION_COL_PRE_ID)?,
            maintainer: sql.get_data(MAINTAINER_COL_PRE_ID)?,
            source_pkg: None, // TODO
            repository: None,
            homepage: sql.get_data(HOMEPAGE_COL_PRE_ID)?,
            arch: String::new(),
            kind: String::new(),
            installed_size: sql.get_data(INSTALLED_SIZE_COL_PRE_ID)?,
            tags: Vec::new(),
            version,
            license: sql.get_data(LICENSE_COL_PRE_ID)?,
            dependencies: Vec::new(),
            suggestions: Vec::new(),
        };

        let kind_id: i64 = sql.get_data(PACKAGE_KIND_ID_COL_PRE_ID)?;
        sql.kill();

        const KIND_ID_COL_PRE_ID: usize = 1;
        let kind_statement = Select::new(
            Some(vec![String::from("kind")]),
            String::from("package_kinds"),
        )
        .where_condition(Where::Equal(KIND_ID_COL_PRE_ID, String::from("id")))
        .to_string();
        let mut sql = db.prepare(kind_statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, KIND_ID_COL_PRE_ID, kind_id);

        const KIND_COL_PRE_ID: usize = 0;
        while let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
            meta.kind = sql.get_data(KIND_COL_PRE_ID)?;
        }

        sql.kill();

        const PACKAGE_ID_COL_PRE_ID: usize = 1;
        let tags_statement = Select::new(
            Some(vec![String::from("tag")]),
            String::from("package_tags"),
        )
        .where_condition(Where::Equal(
            PACKAGE_ID_COL_PRE_ID,
            String::from("package_id"),
        ))
        .to_string();
        let mut sql = db.prepare(tags_statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, PACKAGE_ID_COL_PRE_ID, id);

        const TAG_COL_PRE_ID: usize = 0;
        while let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
            meta.tags.push(sql.get_data(TAG_COL_PRE_ID)?);
        }

        sql.kill();

        let files_statement = Select::new(None, String::from("files"))
            .where_condition(Where::Equal(
                PACKAGE_ID_COL_PRE_ID,
                String::from("package_id"),
            ))
            .to_string();
        let mut sql = db.prepare(files_statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, PACKAGE_ID_COL_PRE_ID, id);

        let mut files: Vec<FileStruct> = Vec::new();

        const PATH_COL_PRE_ID: usize = 2;
        const CHECKSUM_KIND_ID_COL_PRE_ID: usize = 4;
        const CHECKSUM_COL_PRE_ID: usize = 3;
        while let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
            let file = FileStruct {
                path: sql.get_data(PATH_COL_PRE_ID)?,
                checksum_algorithm: get_string_value_by_id(
                    db,
                    String::from("checksum_kinds"),
                    String::from("kind"),
                    sql.get_data(CHECKSUM_KIND_ID_COL_PRE_ID)?,
                )?,
                checksum: sql.get_data(CHECKSUM_COL_PRE_ID)?,
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
        Ok(PkgDataFromDb {
            pkg_id: id,
            meta_dir,
        })
    }

    fn delete_from_db<'lpkg>(&self, db: &Database) -> Result<(), LpmError<PackageError>> {
        const NAME_COL_PRE_ID: usize = 1;
        let statement = Delete::new(String::from("packages"))
            .where_condition(Where::Equal(NAME_COL_PRE_ID, String::from("name")))
            .to_string();

        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, NAME_COL_PRE_ID, self.meta_dir.meta.name.clone());
        try_execute_prepared!(
            sql,
            simple_e_fmt!("Error on deleting package \"{}\".", self.meta_dir.meta.name)
        );
        sql.kill();

        Ok(())
    }
}

fn delete_pkg_files(
    db: &Database,
    pkg_id: i64,
) -> Result<PreparedStatementStatus, LpmError<SqlError>> {
    const PKG_ID_COL_PRE_ID: usize = 1;

    let statement = Delete::new(String::from("files"))
        .where_condition(Where::Equal(PKG_ID_COL_PRE_ID, String::from("package_id")))
        .to_string();

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, PKG_ID_COL_PRE_ID, pkg_id);

    let status = try_execute_prepared!(
        sql,
        simple_e_fmt!("Could not delete from 'files' for package_id {}.", pkg_id)
    );

    sql.kill();

    Ok(status)
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

        const NAME_COL_PRE_ID: usize = 1;
        const ABSOLUTE_PATH_COL_PRE_ID: usize = 2;
        const CHECKSUM_COL_PRE_ID: usize = 3;
        const CHECKSUM_KIND_ID_COL_PRE_ID: usize = 4;
        const PACKAGE_ID_COL_PRE_ID: usize = 5;

        let file_columns = vec![
            Column::new(String::from("name"), NAME_COL_PRE_ID),
            Column::new(String::from("absolute_path"), ABSOLUTE_PATH_COL_PRE_ID),
            Column::new(String::from("checksum"), CHECKSUM_COL_PRE_ID),
            Column::new(
                String::from("checksum_kind_id"),
                CHECKSUM_KIND_ID_COL_PRE_ID,
            ),
            Column::new(String::from("package_id"), PACKAGE_ID_COL_PRE_ID),
        ];
        let statement = Insert::new(Some(file_columns), String::from("files")).to_string();

        let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

        try_bind_val!(
            sql,
            NAME_COL_PRE_ID,
            file_path.file_name().unwrap().to_str().unwrap()
        );
        try_bind_val!(sql, ABSOLUTE_PATH_COL_PRE_ID, format!("/{}", &file.path));
        try_bind_val!(sql, CHECKSUM_COL_PRE_ID, &*file.checksum);
        try_bind_val!(sql, CHECKSUM_KIND_ID_COL_PRE_ID, checksum_id.unwrap());
        try_bind_val!(sql, PACKAGE_ID_COL_PRE_ID, pkg_id);

        try_execute_prepared!(sql, simple_e_fmt!("Could not insert to \"files\" table."));

        sql.kill();
    }

    Ok(())
}

fn is_package_exists(db: &Database, name: &str) -> Result<bool, LpmError<SqlError>> {
    const NAME_COL_PRE_ID: usize = 1;
    let exists_statement = Select::new(None, String::from("packages"))
        .where_condition(Where::Equal(NAME_COL_PRE_ID, String::from("name")))
        .exists()
        .to_string();

    let mut sql = db.prepare(exists_statement.clone(), super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, NAME_COL_PRE_ID, name);

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
    const COL_PRE_ID: usize = 1;
    let get_id_statement = Select::new(Some(vec![String::from("id")]), table)
        .where_condition(Where::Equal(COL_PRE_ID, column))
        .to_string();

    let mut sql = db.prepare(get_id_statement.clone(), super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, COL_PRE_ID, value);

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
    const COL_PRE_ID: usize = 1;
    let statement = Select::new(Some(vec![column]), table)
        .where_condition(Where::Equal(COL_PRE_ID, String::from("id")))
        .to_string();

    let mut sql = db.prepare(statement.clone(), super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, COL_PRE_ID, id);
    try_execute_prepared!(
        sql,
        simple_e_fmt!("Select query failed. SQL:\n {}", statement)
    );

    let result = sql.get_data::<String>(0)?;
    sql.kill();

    Ok(result)
}

pub fn delete_pkg_tags(
    db: &Database,
    pkg_id: i64,
) -> Result<PreparedStatementStatus, LpmError<SqlError>> {
    enable_foreign_keys(db)?;

    const PKG_ID_COL_PRE_ID: usize = 1;

    let statement = Delete::new(String::from("package_tags"))
        .where_condition(Where::Equal(PKG_ID_COL_PRE_ID, String::from("package_id")))
        .to_string();

    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, PKG_ID_COL_PRE_ID, pkg_id);

    let status = try_execute_prepared!(
        sql,
        simple_e_fmt!("Could not delete from 'tags' for package_id {}.", pkg_id)
    );

    sql.kill();

    Ok(status)
}

/// Batch insert of package tags on sqlite
fn insert_pkg_tags(
    db: &Database,
    pkg_id: i64,
    tags: Vec<String>,
) -> Result<PreparedStatementStatus, LpmError<SqlError>> {
    const TAG_COL_PRE_ID: usize = 1;
    const PACKAGE_ID_COL_PRE_ID: usize = 255;
    let package_tag_columns = vec![
        Column::new(String::from("tag"), TAG_COL_PRE_ID),
        Column::new(String::from("package_id"), PACKAGE_ID_COL_PRE_ID),
    ];

    let mut sql_builder = Insert::new(Some(package_tag_columns), String::from("package_tags"));

    for (index, _) in tags.iter().enumerate() {
        let index = index + 1;
        if index == PACKAGE_ID_COL_PRE_ID || index == TAG_COL_PRE_ID {
            continue;
        }

        sql_builder = sql_builder.insert_another_row(vec![index, PACKAGE_ID_COL_PRE_ID]);
    }

    let statement = sql_builder.to_string();
    let mut sql = db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    for (index, tag) in tags.iter().enumerate() {
        let index = index + 1;
        try_bind_val!(sql, index, &**tag);
        try_bind_val!(sql, PACKAGE_ID_COL_PRE_ID, pkg_id);
    }

    let tags = tags.join(", ");
    debug!("Inserting tags {} for package_id: {}", tags, pkg_id);

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
    const KIND_COL_PRE_ID: usize = 1;
    let package_kind_columns = vec![Column::new(String::from("kind"), KIND_COL_PRE_ID)];
    let mut sql_builder = Insert::new(Some(package_kind_columns), String::from("package_kinds"));

    for (index, _) in kinds.iter().enumerate() {
        let index = index + 1;
        if index == KIND_COL_PRE_ID {
            continue;
        }

        sql_builder = sql_builder.insert_another_row(vec![index]);
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
        pre_ids.push(index + 1);
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
