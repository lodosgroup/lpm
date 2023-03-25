use common::config::{LpmConfig, CONFIG_PATH};
use common::pkg::{PkgDataFromDb, PkgDataFromFs};
use common::{log_and_panic, try_or_error, ParserTasks};
use core::*;
use db::init_db;
use db::pkg::DbOpsForInstalledPkg;
use db::{pkg::delete_pkg_kinds, pkg::insert_pkg_kinds, DB_PATH};
#[allow(unused_imports)]
use ehandle::{lpm::LpmError, MainError};
use min_sqlite3_sys::prelude::*;
use std::env;
use std::path::Path;
use term::{info, success};

fn main() {
    try_or_error!(init_db());

    let args: Vec<String> = env::args().collect();
    // TODO
    // this is only for early development testing,
    // will have proper cli parser later
    let cli = |arg: &str| -> Result<(), LpmError<MainError>> {
        match arg {
            "--install" => {
                let path = args.get(2).expect("Package path is missing.");
                info!("Package installation started for {}", path);
                try_or_error!(PkgDataFromFs::start_install_task(path));
                success!("Operation successfully completed.");
            }
            "--update" => {
                let name = args.get(2).expect("Package name is missing.");
                let path = args.get(3).expect("Package path is missing.");
                let db = Database::open(Path::new(DB_PATH))?;
                let mut old_pkg = try_or_error!(PkgDataFromDb::load(&db, name));
                db.close();

                let mut requested_pkg = PkgDataFromFs::start_extract_task(Path::new(path))?;

                info!("Package update started for {}", name);
                old_pkg.start_update_task(&mut requested_pkg)?;
                success!("Operation successfully completed.");
            }
            "--delete" => {
                let name = args.get(2).expect("Package name is missing.");
                let db = Database::open(Path::new(DB_PATH))?;
                let pkg = try_or_error!(PkgDataFromDb::load(&db, name));
                db.close();

                info!("Package deletion started for {}", name);
                try_or_error!(pkg.start_delete_task());
                success!("Operation successfully completed.");
            }
            "--add-pkg-kind" => {
                let db = try_or_error!(Database::open(Path::new(DB_PATH)));
                let kinds: &[String] = &args[2..];
                if kinds.is_empty() {
                    log_and_panic!("Missing value.");
                    // TODO
                    // Show example usage
                }
                info!("Inserting list of package kinds: {:?}", kinds);
                try_or_error!(insert_pkg_kinds(&db, kinds.to_vec()));
                db.close();
                success!("Operation successfully completed.");
            }
            "--delete-pkg-kind" => {
                let db = try_or_error!(Database::open(Path::new(DB_PATH)));
                let kinds: &[String] = &args[2..];
                if kinds.is_empty() {
                    log_and_panic!("Missing value.");
                    // TODO
                    // Show example usage
                }
                info!("Deleting list of package kinds: {:?}", kinds);
                try_or_error!(delete_pkg_kinds(&db, kinds.to_vec()));
                db.close();
                success!("Operation successfully completed.");
            }
            "--plugin" => {
                let plugin_name = args.get(2).expect("Plugin name is missing.");
                let lpm_config = LpmConfig::deserialize(CONFIG_PATH);
                let plugin = lpm_config
                    .plugins
                    .iter()
                    .find(|p| p.name == *plugin_name)
                    .unwrap_or_else(|| panic!("Plugin '{}' not found", plugin_name));

                let plugin_controller = PluginController::load(&plugin.dylib_path)?;
                info!("Plugin '{}' loaded.", plugin_name);
                plugin_controller.run()?;
                plugin_controller.unload();
                info!("Plugin '{}' finished running and unloaded.", plugin_name);
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
