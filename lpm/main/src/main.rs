use cli_parser::{Command, InstallSubcommand, KindSubcommand, ModuleSubcommand};
use common::{log_and_panic, some_or_error};
use core::*;
use std::env;

macro_rules! try_or_error {
    ($fn: expr) => {
        match $fn {
            Result::Ok(val) => val,
            Result::Err(err) => {
                logger::error!("{:?}", err);
                // Terminate app with panic code
                std::process::exit(101);
            }
        }
    };
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match Command::parse_args(&args) {
        Command::Install(pkg_name_or_filepath, subcommand) => match subcommand {
            InstallSubcommand::Local => {
                try_or_error!(install_lod(pkg_name_or_filepath))
            }
            InstallSubcommand::None => todo!(),
        },

        Command::Update(pkg_name, lod_path) => match lod_path {
            Some(lod_path) => try_or_error!(update_lod(pkg_name, lod_path)),
            None => todo!(),
        },

        Command::Delete(pkg_name) => try_or_error!(delete_lod(pkg_name)),

        Command::Kind(subcommand) => match subcommand {
            KindSubcommand::Add(kinds) => {
                let kinds: Vec<String> = kinds.iter().map(|t| t.to_string()).collect();
                try_or_error!(add_pkg_kinds(&kinds))
            }
            KindSubcommand::Delete(kinds) => {
                let kinds: Vec<String> = kinds.iter().map(|t| t.to_string()).collect();
                try_or_error!(delete_pkg_kinds(&kinds))
            }
            KindSubcommand::None => {
                log_and_panic!("Invalid argument on 'lpm --kind'.");
            }
        },

        Command::Module(subcommand) => match subcommand {
            ModuleSubcommand::None => {
                try_or_error!(trigger_lpm_module(args.clone()))
            }
            ModuleSubcommand::Add(list) => {
                let (module_name, dylib_path) = (
                    some_or_error!(list.first(), "Module name is missing"),
                    some_or_error!(list.get(1), "Dynamic library path is missing"),
                );
                try_or_error!(add_module(module_name, dylib_path))
            }
            ModuleSubcommand::Delete(module_names) => {
                let module_names: Vec<String> =
                    module_names.iter().map(|t| t.to_string()).collect();
                try_or_error!(delete_modules(&module_names))
            }
            ModuleSubcommand::List => try_or_error!(print_modules()),
        },

        Command::Configure => try_or_error!(configure()),

        Command::None => {
            log_and_panic!("Invalid argument on 'lpm'.");
        }
    }
}
