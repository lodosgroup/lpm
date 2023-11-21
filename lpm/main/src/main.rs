use cli_parser::{CliParser, Command, ModuleSubcommand, RepositorySubcommand, UpdateSubcommand};
use common::some_or_error;
use core::*;
use std::{env, panic};

macro_rules! try_or_error {
    ($fn: expr) => {
        match $fn {
            Result::Ok(val) => val,
            Result::Err(err) => {
                logger::error!("{:?}", err);
                std::process::exit(101);
            }
        }
    };
}

const LPM_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    panic::set_hook(Box::new(|info| logger::error!("{info}")));

    // TODO
    // get executed command and print it on `cmd::None`

    let core_db = || try_or_error!(open_core_db_connection());

    let args: Vec<String> = env::args().collect();
    let cli_parser = CliParser::parse_args(&args);
    let ctx = || try_or_error!(Ctx::new_from_cli_parser(&cli_parser));

    if cli_parser.commands.is_empty() {
        Command::Help.print_help();
    }

    let mut should_print_green_message = false;
    cli_parser
        .commands
        .iter()
        .for_each(|command| match command {
            Command::Install(args) => {
                should_print_green_message = true;

                if args.print_help {
                    should_print_green_message = false;
                    command.print_help();
                }

                try_or_error!(install_package(ctx(), args));
            }

            Command::Update(pkg_name, subcommands) => {
                should_print_green_message = true;

                if subcommands.is_empty() {
                    if let Some(pkg_name) = pkg_name {
                        try_or_error!(update_pkg_from_repository(ctx(), pkg_name));
                    } else {
                        try_or_error!(update_database_migrations());
                        try_or_error!(get_and_apply_repository_patches(&core_db()));
                        try_or_error!(update_pkgs_from_repository(ctx()));
                    }
                }

                for subcommand in subcommands {
                    match subcommand {
                        UpdateSubcommand::Local(lod_path) => {
                            try_or_error!(update_pkg_from_lod_file(
                                ctx(),
                                pkg_name.expect("Package name is missing."),
                                lod_path
                            ))
                        }
                        UpdateSubcommand::Index => {
                            try_or_error!(get_and_apply_repository_patches(&core_db()))
                        }
                        UpdateSubcommand::Db => try_or_error!(update_database_migrations()),
                        UpdateSubcommand::Packages => {
                            try_or_error!(update_pkgs_from_repository(ctx()))
                        }
                        UpdateSubcommand::All => {
                            try_or_error!(update_database_migrations());
                            try_or_error!(get_and_apply_repository_patches(&core_db()));
                            try_or_error!(update_pkgs_from_repository(ctx()));
                        }

                        UpdateSubcommand::Help => {
                            should_print_green_message = false;
                            command.print_help();
                        }

                        UpdateSubcommand::None => {
                            panic!("Invalid command on 'lpm --update'.");
                        }
                    }
                }
            }

            Command::Delete(pkg_name) => {
                should_print_green_message = true;
                if ["-h", "--help"].contains(pkg_name) {
                    should_print_green_message = false;
                    command.print_help();
                } else {
                    try_or_error!(delete_lod(ctx(), pkg_name));
                }
            }

            Command::Module(subcommand) => match subcommand {
                ModuleSubcommand::None => {
                    try_or_error!(trigger_lpm_module(&core_db(), args.clone()))
                }

                ModuleSubcommand::Add(list) => {
                    should_print_green_message = true;
                    let (module_name, dylib_path) = (
                        some_or_error!(list.first(), "Module name is missing"),
                        some_or_error!(list.get(1), "Dynamic library path is missing"),
                    );
                    try_or_error!(add_module(ctx(), module_name, dylib_path))
                }

                ModuleSubcommand::Delete(module_names) => {
                    should_print_green_message = true;
                    let module_names: Vec<String> =
                        module_names.iter().map(|t| t.to_string()).collect();
                    try_or_error!(delete_modules(ctx(), &module_names))
                }

                ModuleSubcommand::Help => {
                    should_print_green_message = false;
                    command.print_help();
                }

                ModuleSubcommand::List => try_or_error!(print_modules(ctx())),
            },

            Command::Repository(subcommand) => match subcommand {
                RepositorySubcommand::Add(args) => {
                    should_print_green_message = true;
                    let (name, address) = (
                        some_or_error!(args.first(), "Repository name is missing"),
                        some_or_error!(args.get(1), "Repository address is missing"),
                    );
                    try_or_error!(add_repository(ctx(), name, address));
                }

                RepositorySubcommand::Delete(repository_names) => {
                    should_print_green_message = true;
                    let repository_names: Vec<String> =
                        repository_names.iter().map(|t| t.to_string()).collect();
                    try_or_error!(delete_repositories(ctx(), &repository_names))
                }

                RepositorySubcommand::List => {
                    try_or_error!(print_repositories(&core_db()))
                }

                RepositorySubcommand::Help => {
                    should_print_green_message = false;
                    command.print_help();
                }

                RepositorySubcommand::None => {
                    panic!("Invalid command on 'lpm --repository'.");
                }
            },

            Command::Help => {
                should_print_green_message = false;
                command.print_help();
            }

            Command::Version => {
                println!("lpm version: {}", LPM_VERSION);
            }
        });

    if should_print_green_message {
        logger::success!("Operation successfully completed.");
    }
}
