use common::{log_and_panic, try_or_error};
use core::*;
use ehandle::{lpm::LpmError, MainError};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    // TODO
    // this is only for early development testing,
    // will have proper cli parser later
    let cli = |arg: &str| -> Result<(), LpmError<MainError>> {
        match arg {
            "--install" => {
                let pkg_path = args.get(2).expect("Package path is missing.");
                install_lod(pkg_path)?;
            }
            "--update" => {
                let pkg_name = args.get(2).expect("Package name is missing.");
                let pkg_path = args.get(3).expect("Package path is missing.");
                update_lod(pkg_name, pkg_path)?;
            }
            "--delete" => {
                let pkg_name = args.get(2).expect("Package name is missing.");
                delete_lod(pkg_name)?;
            }
            "--add-pkg-kinds" => {
                let kinds: &[String] = &args[2..];
                add_pkg_kinds(kinds)?;
            }
            "--delete-pkg-kinds" => {
                let kinds: &[String] = &args[2..];
                delete_pkg_kinds(kinds)?;
            }
            "--add-module" => {
                let module_name = args.get(2).expect("Module name is missing.");
                let dylib_path = args.get(3).expect("Dynamic library path is missing.");

                add_module(module_name, dylib_path)?;
            }
            "--delete-modules" => {
                let module_names: &[String] = &args[2..];
                delete_modules(module_names)?;
            }
            "--modules" => {
                print_modules()?;
            }

            "--module" => trigger_lpm_module(args.clone())?,

            "--configure" => configure()?,

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
