use common::{log_and_panic, some_or_error};
use core::*;
use std::env;

mod cli;

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
    match cli::parse_args(&args) {
        cli::Command::Install(pkg_name_or_filepath, subcommand) => match subcommand {
            cli::Subcommand::Local => try_or_error!(install_lod(pkg_name_or_filepath)),
            cli::Subcommand::Add(_) => todo!(),
            cli::Subcommand::Delete(_) => todo!(),
            cli::Subcommand::List => todo!(),
            cli::Subcommand::None => todo!(),
        },

        cli::Command::Update(pkg_name, lod_path) => match lod_path {
            Some(lod_path) => try_or_error!(update_lod(pkg_name, lod_path)),
            None => todo!(),
        },

        cli::Command::Delete(pkg_name) => try_or_error!(delete_lod(pkg_name)),

        cli::Command::Kind(subcommand) => match subcommand {
            cli::Subcommand::Add(kinds) => {
                let kinds: Vec<String> = kinds.iter().map(|t| t.to_string()).collect();
                try_or_error!(add_pkg_kinds(&kinds))
            }
            cli::Subcommand::Delete(kinds) => {
                let kinds: Vec<String> = kinds.iter().map(|t| t.to_string()).collect();
                try_or_error!(delete_pkg_kinds(&kinds))
            }
            cli::Subcommand::List | cli::Subcommand::Local | cli::Subcommand::None => {
                log_and_panic!("Invalid argument on 'lpm --kind'.");
            }
        },

        cli::Command::Module(subcommand) => match subcommand {
            cli::Subcommand::None | cli::Subcommand::Local => {
                try_or_error!(trigger_lpm_module(args.clone()))
            }
            cli::Subcommand::Add(list) => {
                let (module_name, dylib_path) = (
                    some_or_error!(list.first(), "Module name is missing"),
                    some_or_error!(list.get(1), "Dynamic library path is missing"),
                );
                try_or_error!(add_module(module_name, dylib_path))
            }
            cli::Subcommand::Delete(module_names) => {
                let module_names: Vec<String> =
                    module_names.iter().map(|t| t.to_string()).collect();
                try_or_error!(delete_modules(&module_names))
            }
            cli::Subcommand::List => try_or_error!(print_modules()),
        },

        cli::Command::Configure => try_or_error!(configure()),

        cli::Command::None => {
            log_and_panic!("Invalid argument on 'lpm'.");
        }
    }
}
