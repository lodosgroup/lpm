use crate::{enable_foreign_keys, transaction_op, Transaction};

use common::meta::FileStruct;
use common::pkg::MetaDir;
use common::pkg::PkgDataFromDb;
use common::pkg::PkgDataFromFs;
use common::version::Condition;
use common::{meta::Meta, version::VersionStruct, Files};
use ehandle::{
    db::SqlError,
    lpm::LpmError,
    pkg::{PackageError, PackageErrorKind},
    simple_e_fmt, try_bind_val, try_execute_prepared, ErrorCommons,
};
use logger::info;
use min_sqlite3_sys::prelude::*;
use sql_builder::delete::*;
use sql_builder::insert::*;
use sql_builder::select::*;
use sql_builder::update::Update;
use sql_builder::Column;
use std::path::Path;
use std::path::PathBuf;

pub trait DbOpsForInstalledPkg {
    fn load(core_db: &Database, name: &str) -> Result<Self, LpmError<PackageError>>
    where
        Self: Sized;
    fn delete_from_db(&self, core_db: &Database) -> Result<(), LpmError<PackageError>>;
}

pub trait DbOpsForBuildFile {
    fn insert_to_db(
        &self,
        core_db: &Database,
        group_id: String,
    ) -> Result<i64, LpmError<PackageError>>;
    fn update_existing_pkg(
        &self,
        core_db: &Database,
        pkg_id: i64,
        new_group_id: String,
    ) -> Result<(), LpmError<PackageError>>;
}

impl DbOpsForBuildFile for PkgDataFromFs {
    fn insert_to_db(
        &self,
        core_db: &Database,
        group_id: String,
    ) -> Result<i64, LpmError<PackageError>> {
        const NAME_COL_PRE_ID: usize = 1;
        const GROUP_ID_COL_PRE_ID: usize = 2;
        const INSTALLED_SIZE_COL_PRE_ID: usize = 3;
        const V_MAJOR_COL_PRE_ID: usize = 4;
        const V_MINOR_COL_PRE_ID: usize = 5;
        const V_PATCH_COL_PRE_ID: usize = 6;
        const V_TAG_COL_PRE_ID: usize = 7;
        const V_READABLE_COL_PRE_ID: usize = 8;

        let package_columns = vec![
            Column::new(String::from("name"), NAME_COL_PRE_ID),
            Column::new(String::from("group_id"), GROUP_ID_COL_PRE_ID),
            Column::new(String::from("installed_size"), INSTALLED_SIZE_COL_PRE_ID),
            Column::new(String::from("v_major"), V_MAJOR_COL_PRE_ID),
            Column::new(String::from("v_minor"), V_MINOR_COL_PRE_ID),
            Column::new(String::from("v_patch"), V_PATCH_COL_PRE_ID),
            Column::new(String::from("v_tag"), V_TAG_COL_PRE_ID),
            Column::new(String::from("v_readable"), V_READABLE_COL_PRE_ID),
        ];

        let statement = Insert::new(Some(package_columns), String::from("packages")).to_string();

        let mut sql = core_db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

        try_bind_val!(sql, NAME_COL_PRE_ID, &*self.meta_dir.meta.name);

        try_bind_val!(sql, GROUP_ID_COL_PRE_ID, &*group_id);

        try_bind_val!(
            sql,
            INSTALLED_SIZE_COL_PRE_ID,
            self.meta_dir.meta.installed_size
        );

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

        let sql_status = sql.execute_prepared();
        if PreparedStatementStatus::Done != sql_status {
            logger::error!(
                "Database sync failed with '{}' package. Sql status: {:?}",
                self.meta_dir.meta.name,
                sql_status
            );

            return Err(
                PackageErrorKind::InstallationFailed(self.meta_dir.meta.name.clone()).to_lpm_err(),
            );
        }

        let pkg_id = super::get_last_insert_row_id(core_db)?;

        match insert_files(core_db, pkg_id, &self.meta_dir.files) {
            Ok(_) => Ok(pkg_id),
            Err(err) => Err(err),
        }
    }

    fn update_existing_pkg(
        &self,
        core_db: &Database,
        pkg_id: i64,
        new_group_id: String,
    ) -> Result<(), LpmError<PackageError>> {
        enable_foreign_keys(core_db)?;

        transaction_op(core_db, Transaction::Begin)?;

        const NAME_COL_PRE_ID: usize = 1;
        const GROUP_ID_COL_PRE_ID: usize = 2;
        const INSTALLED_SIZE_COL_PRE_ID: usize = 3;
        const V_MAJOR_COL_PRE_ID: usize = 4;
        const V_MINOR_COL_PRE_ID: usize = 5;
        const V_PATCH_COL_PRE_ID: usize = 6;
        const V_TAG_COL_PRE_ID: usize = 7;
        const V_READABLE_COL_PRE_ID: usize = 8;

        let update_fields = vec![
            Column::new(String::from("group_id"), GROUP_ID_COL_PRE_ID),
            Column::new(String::from("installed_size"), INSTALLED_SIZE_COL_PRE_ID),
            Column::new(String::from("v_major"), V_MAJOR_COL_PRE_ID),
            Column::new(String::from("v_minor"), V_MINOR_COL_PRE_ID),
            Column::new(String::from("v_patch"), V_PATCH_COL_PRE_ID),
            Column::new(String::from("v_tag"), V_TAG_COL_PRE_ID),
            Column::new(String::from("v_readable"), V_READABLE_COL_PRE_ID),
        ];

        let statement = Update::new(update_fields, String::from("packages"))
            .where_condition(Where::Equal(NAME_COL_PRE_ID, String::from("name")))
            .to_string();

        let mut sql = core_db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

        try_bind_val!(sql, NAME_COL_PRE_ID, &*self.meta_dir.meta.name);

        // TODO
        // Update all of old group_ids to new one
        try_bind_val!(sql, GROUP_ID_COL_PRE_ID, &*new_group_id);

        try_bind_val!(
            sql,
            INSTALLED_SIZE_COL_PRE_ID,
            self.meta_dir.meta.installed_size
        );

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
            transaction_op(core_db, Transaction::Rollback)?;

            return Err(
                PackageErrorKind::InstallationFailed(self.meta_dir.meta.name.clone()).to_lpm_err(),
            );
        }

        match delete_pkg_files(core_db, pkg_id) {
            Ok(_) => (),
            Err(err) => {
                transaction_op(core_db, Transaction::Rollback)?;
                return Err(err)?;
            }
        };

        match insert_files(core_db, pkg_id, &self.meta_dir.files) {
            Ok(_) => Ok(()),
            Err(err) => {
                transaction_op(core_db, Transaction::Rollback)?;
                Err(err)
            }
        }
    }
}

impl DbOpsForInstalledPkg for PkgDataFromDb {
    fn load(core_db: &Database, name: &str) -> Result<Self, LpmError<PackageError>> {
        info!("Loading '{}' from database..", name);

        const PKG_ID_COL_PRE_ID: usize = 0;
        const NAME_COL_PRE_ID: usize = 1;
        const GROUP_ID_COL_PRE_ID: usize = 2;
        const INSTALLED_SIZE_COL_PRE_ID: usize = 3;
        const V_MAJOR_COL_PRE_ID: usize = 4;
        const V_MINOR_COL_PRE_ID: usize = 5;
        const V_PATCH_COL_PRE_ID: usize = 6;
        const V_TAG_COL_PRE_ID: usize = 7;
        const V_READABLE_COL_PRE_ID: usize = 8;

        let statement = Select::new(None, String::from("packages"))
            .where_condition(Where::Equal(NAME_COL_PRE_ID, String::from("name")))
            .to_string();
        let mut sql = core_db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, NAME_COL_PRE_ID, name);
        try_execute_prepared!(
            sql,
            simple_e_fmt!("Error SELECT query on 'packages' table.")
        );

        let id: i64 = sql.get_data(PKG_ID_COL_PRE_ID).unwrap_or(0);

        if id == 0 {
            return Err(PackageErrorKind::DoesNotExists(name.to_string()).to_lpm_err());
        }

        let group_id = sql.get_data(GROUP_ID_COL_PRE_ID)?;

        let version = VersionStruct {
            major: sql.get_data(V_MAJOR_COL_PRE_ID)?,
            minor: sql.get_data(V_MINOR_COL_PRE_ID)?,
            patch: sql.get_data(V_PATCH_COL_PRE_ID)?,
            tag: sql.get_data(V_TAG_COL_PRE_ID)?,
            readable_format: sql.get_data(V_READABLE_COL_PRE_ID)?,
            condition: Condition::default(),
        };

        let meta = Meta {
            name: sql.get_data(NAME_COL_PRE_ID)?,
            arch: String::new(),
            installed_size: sql.get_data(INSTALLED_SIZE_COL_PRE_ID)?,
            version,
            dependencies: Vec::new(),
            suggestions: Vec::new(),
        };

        const PACKAGE_ID_COL_PRE_ID: usize = 1;

        let files_statement = Select::new(None, String::from("files"))
            .where_condition(Where::Equal(
                PACKAGE_ID_COL_PRE_ID,
                String::from("package_id"),
            ))
            .to_string();
        let mut sql = core_db.prepare(files_statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(sql, PACKAGE_ID_COL_PRE_ID, id);

        let mut files: Vec<FileStruct> = Vec::new();

        const PATH_COL_PRE_ID: usize = 2;
        const CHECKSUM_COL_PRE_ID: usize = 3;
        const CHECKSUM_ALGORITHM_COL_PRE_ID: usize = 4;
        while let PreparedStatementStatus::FoundRow = sql.execute_prepared() {
            let file = FileStruct {
                path: sql.get_data(PATH_COL_PRE_ID)?,
                checksum_algorithm: sql.get_data(CHECKSUM_ALGORITHM_COL_PRE_ID)?,
                checksum: sql.get_data(CHECKSUM_COL_PRE_ID)?,
            };

            files.push(file);
        }

        let files = Files(files);
        let meta_fields = MetaDir {
            path: PathBuf::default(),
            meta,
            files,
        };

        info!("Package '{}' successfully loaded.", name);
        Ok(PkgDataFromDb {
            pkg_id: id,
            group_id,
            meta_fields,
        })
    }

    fn delete_from_db<'lpkg>(&self, core_db: &Database) -> Result<(), LpmError<PackageError>> {
        const GROUP_ID_COL_PRE_ID: usize = 1;
        let statement = Delete::new(String::from("packages"))
            .where_condition(Where::Equal(GROUP_ID_COL_PRE_ID, String::from("group_id")))
            .to_string();

        let mut sql = core_db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;
        try_bind_val!(
            sql,
            GROUP_ID_COL_PRE_ID,
            self.meta_fields.meta.get_group_id()
        );
        try_execute_prepared!(
            sql,
            simple_e_fmt!(
                "Error on deleting package \"{}\".",
                self.meta_fields.meta.name
            )
        );

        Ok(())
    }
}

fn delete_pkg_files(
    core_db: &Database,
    pkg_id: i64,
) -> Result<PreparedStatementStatus, LpmError<SqlError>> {
    const PKG_ID_COL_PRE_ID: usize = 1;

    let statement = Delete::new(String::from("files"))
        .where_condition(Where::Equal(PKG_ID_COL_PRE_ID, String::from("package_id")))
        .to_string();

    let mut sql = core_db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, PKG_ID_COL_PRE_ID, pkg_id);

    let status = try_execute_prepared!(
        sql,
        simple_e_fmt!("Could not delete from 'files' for package_id {}.", pkg_id)
    );

    Ok(status)
}

fn insert_files(
    core_db: &Database,
    pkg_id: i64,
    files: &Files,
) -> Result<(), LpmError<PackageError>> {
    let files = &files.0;

    for file in files {
        let file_path = Path::new(&file.path);

        const NAME_COL_PRE_ID: usize = 1;
        const ABSOLUTE_PATH_COL_PRE_ID: usize = 2;
        const CHECKSUM_COL_PRE_ID: usize = 3;
        const CHECKSUM_ALGORITHM_COL_PRE_ID: usize = 4;
        const PACKAGE_ID_COL_PRE_ID: usize = 5;

        let file_columns = vec![
            Column::new(String::from("name"), NAME_COL_PRE_ID),
            Column::new(String::from("absolute_path"), ABSOLUTE_PATH_COL_PRE_ID),
            Column::new(String::from("checksum"), CHECKSUM_COL_PRE_ID),
            Column::new(
                String::from("checksum_algorithm"),
                CHECKSUM_ALGORITHM_COL_PRE_ID,
            ),
            Column::new(String::from("package_id"), PACKAGE_ID_COL_PRE_ID),
        ];
        let statement = Insert::new(Some(file_columns), String::from("files")).to_string();

        let mut sql = core_db.prepare(statement, super::SQL_NO_CALLBACK_FN)?;

        try_bind_val!(
            sql,
            NAME_COL_PRE_ID,
            file_path.file_name().unwrap().to_str().unwrap()
        );
        try_bind_val!(sql, ABSOLUTE_PATH_COL_PRE_ID, format!("/{}", &file.path));
        try_bind_val!(sql, CHECKSUM_COL_PRE_ID, &*file.checksum);
        try_bind_val!(
            sql,
            CHECKSUM_ALGORITHM_COL_PRE_ID,
            &*file.checksum_algorithm
        );
        try_bind_val!(sql, PACKAGE_ID_COL_PRE_ID, pkg_id);

        try_execute_prepared!(sql, simple_e_fmt!("Could not insert to \"files\" table."));
    }

    Ok(())
}

pub fn is_package_exists(core_db: &Database, name: &str) -> Result<bool, LpmError<SqlError>> {
    const NAME_COL_PRE_ID: usize = 1;
    let exists_statement = Select::new(None, String::from("packages"))
        .where_condition(Where::Equal(NAME_COL_PRE_ID, String::from("name")))
        .exists()
        .to_string();

    let mut sql = core_db.prepare(exists_statement.clone(), super::SQL_NO_CALLBACK_FN)?;

    try_bind_val!(sql, NAME_COL_PRE_ID, name);

    try_execute_prepared!(
        sql,
        simple_e_fmt!("Select exists query failed. SQL:\n {}", exists_statement)
    );

    let result = sql.get_data::<i64>(0).unwrap_or(0);

    Ok(result == 1)
}
