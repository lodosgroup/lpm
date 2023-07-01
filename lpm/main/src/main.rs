use cli_parser::{CliParser, Command, InstallSubcommand, ModuleSubcommand, UpdateSubcommand};
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

fn main() {
    panic::set_hook(Box::new(|info| logger::error!("{info}")));

    // TODO
    // get executed command and print it on `cmd::None`

    let core_db = || try_or_error!(open_core_db_connection());

    let args: Vec<String> = env::args().collect();
    let cli_parser = CliParser::parse_args(&args);
    let ctx = || try_or_error!(Ctx::new_from_cli_parser(&cli_parser));

    if cli_parser.commands.is_empty() {
        panic!("Invalid command on 'lpm'.");
    }

    cli_parser
        .commands
        .iter()
        .for_each(|command| match command {
            Command::Install(pkg_name_or_filepath, subcommand) => match subcommand {
                InstallSubcommand::Local => {
                    try_or_error!(install_from_lod_file(ctx(), pkg_name_or_filepath))
                }

                InstallSubcommand::None => {
                    try_or_error!(install_from_repository(ctx(), pkg_name_or_filepath,))
                }
            },

            Command::Update(pkg_name, subcommands) => {
                if subcommands.is_empty() {
                    try_or_error!(update_from_repository(
                        ctx(),
                        pkg_name.expect("Package name is missing."),
                    ));
                }

                for subcommand in subcommands {
                    match subcommand {
                        UpdateSubcommand::Local(lod_path) => {
                            try_or_error!(update_from_lod_file(
                                ctx(),
                                pkg_name.expect("Package name is missing."),
                                lod_path
                            ))
                        }
                        UpdateSubcommand::Index => {
                            try_or_error!(get_and_apply_repository_patches(&core_db()))
                        }
                        UpdateSubcommand::Db => try_or_error!(update_database_migrations()),
                        UpdateSubcommand::Packages => todo!(),
                        UpdateSubcommand::All => {
                            try_or_error!(update_database_migrations());
                            try_or_error!(get_and_apply_repository_patches(&core_db()))
                        }
                        UpdateSubcommand::None => {
                            panic!("Invalid command on 'lpm --update'.");
                        }
                    }
                }
            }

            Command::Delete(pkg_name) => try_or_error!(delete_lod(ctx(), pkg_name)),

            Command::Module(subcommand) => match subcommand {
                ModuleSubcommand::None => {
                    try_or_error!(trigger_lpm_module(&core_db(), args.clone()))
                }
                ModuleSubcommand::Add(list) => {
                    let (module_name, dylib_path) = (
                        some_or_error!(list.first(), "Module name is missing"),
                        some_or_error!(list.get(1), "Dynamic library path is missing"),
                    );
                    try_or_error!(add_module(ctx(), module_name, dylib_path))
                }
                ModuleSubcommand::Delete(module_names) => {
                    let module_names: Vec<String> =
                        module_names.iter().map(|t| t.to_string()).collect();
                    try_or_error!(delete_modules(ctx(), &module_names))
                }
                ModuleSubcommand::List => try_or_error!(print_modules(ctx())),
            },

            Command::Repository(subcommand) => match subcommand {
                cli_parser::RepositorySubcommand::Add(args) => {
                    let (name, address) = (
                        some_or_error!(args.first(), "Repository name is missing"),
                        some_or_error!(args.get(1), "Repository address is missing"),
                    );
                    try_or_error!(add_repository(ctx(), name, address));
                }
                cli_parser::RepositorySubcommand::Delete(repository_names) => {
                    let repository_names: Vec<String> =
                        repository_names.iter().map(|t| t.to_string()).collect();
                    try_or_error!(delete_repositories(ctx(), &repository_names))
                }
                cli_parser::RepositorySubcommand::List => {
                    try_or_error!(print_repositories(&core_db()))
                }
                cli_parser::RepositorySubcommand::None => {
                    panic!("Invalid command on 'lpm --repository'.");
                }
            },
        });

    logger::success!("Operation successfully completed.");
}
