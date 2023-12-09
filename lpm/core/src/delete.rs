use crate::{
    stage1::{get_scripts, Stage1Tasks, PKG_SCRIPTS_DIR},
    Ctx,
};

use cli_parser::DeleteArgs;
use common::{
    ctx_confirmation_check,
    pkg::{PkgDataFromDb, ScriptPhase},
};
use db::{
    enable_core_db_wal1, enable_foreign_keys, pkg::DbOpsForInstalledPkg, transaction_op,
    Transaction,
};
use ehandle::{lpm::LpmError, pkg::PackageErrorKind, ErrorCommons, MainError};
use logger::{info, warning};
use min_sqlite3_sys::prelude::Database;
use std::{fs, path::Path, sync::Arc, thread};

trait PkgDeleteTasks {
    fn start_delete_task(&self, core_db: &Database) -> Result<(), LpmError<MainError>>;
}

impl PkgDeleteTasks for PkgDataFromDb {
    fn start_delete_task(&self, core_db: &Database) -> Result<(), LpmError<MainError>> {
        // Enable constraits to remove records that are related with package
        enable_foreign_keys(core_db)?;

        transaction_op(core_db, Transaction::Begin)?;

        let pkg_lib_dir = Path::new(PKG_SCRIPTS_DIR).join(&self.meta_fields.meta.name);
        let scripts = get_scripts(&pkg_lib_dir.join("scripts"))?;

        if let Err(err) = scripts.execute_script(vec![], ScriptPhase::PreDelete) {
            transaction_op(core_db, Transaction::Rollback)?;
            return Err(err);
        }

        info!("Syncing with package database..");
        if self.delete_from_db(core_db).is_err() {
            transaction_op(core_db, Transaction::Rollback)?;

            return Err(
                PackageErrorKind::DeletionFailed(self.meta_fields.meta.name.clone()).to_lpm_err(),
            )?;
        };

        info!("Deleting package files from system..");
        for file in &self.meta_fields.files.0 {
            if Path::new(&file.path).exists() {
                fs::remove_file(&file.path)?;
            } else {
                warning!("Path -> {} <- is not exists", file.path);
            }
        }

        if Path::new(&pkg_lib_dir).exists() {
            fs::remove_dir_all(pkg_lib_dir)?;
        }

        if let Err(err) = scripts.execute_script(vec![], ScriptPhase::PostDelete) {
            transaction_op(core_db, Transaction::Rollback)?;
            return Err(err);
        }

        transaction_op(core_db, Transaction::Commit)?;
        info!("Deletion transaction completed.");

        Ok(())
    }
}

pub fn delete_packages(ctx: Ctx, args: &DeleteArgs) -> Result<(), LpmError<MainError>> {
    enable_core_db_wal1(&ctx.core_db)?;

    let mut pkgs = vec![];
    for pkg_name in &args.packages {
        pkgs.push(PkgDataFromDb::load(&ctx.core_db, pkg_name)?);
    }

    {
        // TODO
        // package size is missing
        // total size is missing
        // use colors
        println!("\nPackage list to be deleted:");
        pkgs.iter().for_each(|pkg| {
            println!("  - {}", pkg.meta_fields.meta.get_group_id());
        });
        println!();
    }

    ctx_confirmation_check!(ctx);

    thread::scope(|s| -> Result<(), LpmError<MainError>> {
        pkgs.iter().for_each(|pkg| {
            let core_db = Arc::new(&ctx.core_db);
            s.spawn(move || -> Result<(), LpmError<MainError>> {
                if pkg.meta_fields.meta.get_group_id() != pkg.group_id {
                    return Err(PackageErrorKind::DependencyOfAnotherPackage {
                        package: pkg.meta_fields.meta.name.clone(),
                        depends_on: pkg.group_id.clone(),
                    }
                    .to_lpm_err())?;
                };

                info!("Package deletion started for {}", pkg.meta_fields.meta.name);
                pkg.start_delete_task(&core_db)?;

                Ok(())
            });
        });

        Ok(())
    })?;

    Ok(())
}
