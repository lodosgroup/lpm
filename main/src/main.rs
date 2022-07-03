use common::pkg::LodPkg;
use core::installation::InstallationTasks;
use db::init_db;
use db::{pkg::delete_pkg_kinds, pkg::insert_pkg_kinds, DB_PATH};
use min_sqlite3_sys::prelude::*;
use std::env;
use std::path::Path;

#[allow(unused_imports)]
use ehandle::{RuntimeError, RuntimeErrorKind};

#[cfg(target_os = "linux")]
fn main() -> Result<(), RuntimeError> {
    use db::pkg::LodPkgCoreDbOps;

    init_db()?;

    let args: Vec<String> = env::args().collect();

    let cli = |arg: &str| -> Result<(), RuntimeError> {
        match arg {
            "--install" => {
                let mut pkg = LodPkg::from_fs(args.get(2).expect("Package path is missing."));
                pkg.start_installation()?;
            }
            "--delete" => {
                let db = Database::open(Path::new(DB_PATH)).unwrap();
                let pkg = LodPkg::from_db(&db, args.get(2).expect("Package name is missing."))?;
                println!("{:?}", pkg);

                // pkg.delete_package()?;
            }
            "--add-pkg-kind" => {
                let db = Database::open(Path::new(DB_PATH))?;
                let kinds = &args[2..];
                insert_pkg_kinds(&db, kinds.to_vec())?;
                db.close();
            }
            "--delete-pkg-kind" => {
                let db = Database::open(Path::new(DB_PATH))?;
                let kinds = &args[2..];
                delete_pkg_kinds(&db, kinds.to_vec())?;
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

#[cfg(not(target_os = "linux"))]
fn main() -> Result<(), RuntimeError> {
    Err(RuntimeErrorKind::UnsupportedPlatform(None).throw())
}
