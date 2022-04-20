use common::pkg::LodPkg;
use core::installation::InstallationTasks;
use db::init_db;
use db::{pkg::delete_pkg_kinds, pkg::insert_pkg_kinds, DB_PATH};
use min_sqlite3_sys::prelude::*;
use std::env;
use std::path::Path;

#[allow(unused_imports)]
use ehandle::{RuntimeError, RuntimeErrorKind};

#[cfg(not(target_os = "linux"))]
compile_error!("LodPM can not be built on non-linux operating systems.");

#[cfg(target_os = "linux")]
fn main() -> Result<(), RuntimeError> {
    init_db()?;

    let args: Vec<String> = env::args().collect();

    let cli = |arg: &str| -> Result<(), RuntimeError> {
        match arg {
            "--install" => {
                let mut pkg = LodPkg::new(args.get(2).expect("Package path is missing."));
                pkg.start_installation()?;
            }
            "--add-pkg-kind" => {
                let db = Database::open(Path::new(DB_PATH))?;
                let kinds = &args[2..];
                insert_pkg_kinds(kinds.to_vec(), &db)?;
                db.close();
            }
            "--delete-pkg-kind" => {
                let db = Database::open(Path::new(DB_PATH))?;
                let kinds = &args[2..];
                delete_pkg_kinds(kinds.to_vec(), &db)?;
                db.close();
            }
            _ => panic!("Invalid argument."),
        };

        Ok(())
    };

    match args.get(1) {
        Some(arg) => cli(arg)?,
        None => panic!("Missing argument"),
    }

    Ok(())
}

// handle with pre-build executions
#[cfg(not(target_os = "linux"))]
fn main() -> Result<(), RuntimeError> {
    Err(RuntimeError::new(RuntimeErrorKind::UnsupportedPlatform))
}
