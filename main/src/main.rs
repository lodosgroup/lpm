use common::pkg::LodPkg;
use core::{deletion::DeletionTasks, installation::InstallationTasks};
use db::init_db;
use db::pkg::LodPkgCoreDbOps;
use db::{pkg::delete_pkg_kinds, pkg::insert_pkg_kinds, DB_PATH};
use min_sqlite3_sys::prelude::*;
use std::env;
use std::path::Path;
use term::error;

#[allow(unused_imports)]
use ehandle::{lpm::LpmError, BuildtimeErrorKind, MainError};

macro_rules! try_or_error {
    ($fn: expr) => {
        match $fn {
            Result::Ok(val) => val,
            Result::Err(err) => {
                error!("{:?}", err);
                // Terminate app with panic code
                process::exit(101);
            }
        }
    };
}

macro_rules! log_and_panic {
    ($log: expr) => {
        error!($log);

        // Terminate app with panic code
        process::exit(101);
    };
}

#[cfg(target_os = "linux")]
fn main() {
    use std::process;

    try_or_error!(init_db());

    let args: Vec<String> = env::args().collect();
    let cli = |arg: &str| -> Result<(), LpmError<MainError>> {
        match arg {
            "--install" => {
                let mut pkg = LodPkg::from_fs(args.get(2).expect("Package path is missing."));
                try_or_error!(pkg.start_installation());
            }
            "--delete" => {
                let db = Database::open(Path::new(DB_PATH)).unwrap();
                let pkg = try_or_error!(LodPkg::from_db(
                    &db,
                    args.get(2).expect("Package name is missing.")
                ));
                db.close();

                try_or_error!(pkg.start_deletion());
            }
            "--add-pkg-kind" => {
                let db = try_or_error!(Database::open(Path::new(DB_PATH)));
                let kinds = &args[2..];
                try_or_error!(insert_pkg_kinds(&db, kinds.to_vec()));
                db.close();
            }
            "--delete-pkg-kind" => {
                let db = try_or_error!(Database::open(Path::new(DB_PATH)));
                let kinds = &args[2..];
                try_or_error!(delete_pkg_kinds(&db, kinds.to_vec()));
                db.close();
            }
            _ => {
                log_and_panic!("Invalid argument.");
            }
        };

        Ok(())
    };

    match args.get(1) {
        Some(arg) => try_or_error!(cli(arg)),
        None => {
            log_and_panic!("Missing argument.");
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn main() -> Result<(), LpmError<MainError>> {
    Err(BuildtimeErrorKind::UnsupportedPlatform(None).throw())
}
