pub use check_path::CheckPathArgs;
pub use delete::DeleteArgs;
pub use install::InstallArgs;
pub use module::ModuleSubcommand;
pub use repository::RepositorySubcommand;
pub use update::UpdateSubcommand;

mod check_path;
mod delete;
mod install;
mod module;
mod repository;
mod update;

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Install(InstallArgs<'a>),
    Update(Option<&'a str>, Vec<UpdateSubcommand<'a>>),
    Delete(DeleteArgs<'a>),
    Module(ModuleSubcommand<'a>),
    Repository(RepositorySubcommand<'a>),
    CheckPath(CheckPathArgs<'a>),
    Version,
    Help,
}

#[derive(Default)]
pub struct CliParser<'a> {
    pub commands: Vec<Command<'a>>,
    pub force_yes: bool,
}

impl Command<'_> {
    pub fn print_help(&self) {
        match self {
            Command::Install(_subcommand) => {
                println!("{}", InstallArgs::help());
            }

            Command::Update(_pkg_name, _subcommands) => {
                println!("{}", UpdateSubcommand::help());
            }

            Command::Delete(_pkg_name) => {
                println!("{}", DeleteArgs::help());
            }

            Command::Module(_subcommand) => {
                println!("{}", ModuleSubcommand::help());
            }

            Command::Repository(_subcommand) => {
                println!("{}", RepositorySubcommand::help());
            }

            Command::CheckPath(_args) => {
                println!("{}", CheckPathArgs::help());
            }

            Command::Help => {
                let help = "Lod Package Manager Command Line Interface

Usage: lpm [SUBCOMMAND] [SUBCOMMAND FLAGS] [SUBCOMMAND OPTIONS]

Subcommands:
    -i, --install                                             Install package to system from remote repository or filesystem
    -d, --delete                                              Delete package from system
    -u, --update                                              Update operations(packages, repository index, lpm database migrations)
    -r, --repository                                          Remote repository operations (add, delete, list)
    -m, --module                                              Dynamic module operations (add, delete, list, run)

    --check-path                                              Check if the target path is owned by any of the lpm packages

For more specific help, go for `lpm [SUBCOMMAND] --help`
";
                println!("{}", help);
            }

            Command::Version => panic!("This should never happen. Seems like a bug."),
        }
    }
}

impl CliParser<'_> {
    pub fn parse_args(args: &[String]) -> CliParser<'_> {
        let mut iter = args.iter().peekable();

        let mut cli_parser = CliParser::default();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--install" | "-i" => {
                    cli_parser
                        .commands
                        .push(Command::Install(InstallArgs::parse(&mut iter)));
                }
                "--update" | "-u" => {
                    let mut pkg_name: Option<&String> = None;
                    let mut subcommands = vec![];

                    if let Some(value) = iter.peek() {
                        if !value.starts_with('-') {
                            pkg_name = iter.next();
                        };
                    }
                    while iter.peek().is_some() {
                        subcommands.push(UpdateSubcommand::parse(&mut iter));
                    }

                    cli_parser
                        .commands
                        .push(Command::Update(pkg_name.map(|t| t.as_str()), subcommands));
                }
                "--delete" | "-d" => {
                    cli_parser
                        .commands
                        .push(Command::Delete(DeleteArgs::parse(&mut iter)));
                }
                "--module" | "-m" => {
                    cli_parser
                        .commands
                        .push(Command::Module(ModuleSubcommand::parse(&mut iter)));
                }
                "--repository" | "-r" => {
                    cli_parser
                        .commands
                        .push(Command::Repository(RepositorySubcommand::parse(&mut iter)));
                }
                "--check-path" => {
                    cli_parser
                        .commands
                        .push(Command::CheckPath(CheckPathArgs::parse(&mut iter)));
                }
                "--yes" | "-y" => {
                    cli_parser.force_yes = true;
                }
                "--version" | "-v" => {
                    cli_parser.commands.push(Command::Version);
                }
                "--help" | "-h" => {
                    cli_parser.commands.push(Command::Help);
                }
                _ => {}
            }
        }

        cli_parser
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::field_reassign_with_default)]

    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_parse_install() {
        {
            let args = vec![String::from("--install")];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);

            let mut args = InstallArgs::default();
            args.print_help = true;

            assert_eq!(cli_parser.commands[0], Command::Install(args));
        }

        {
            let args = vec![
                String::from("--install"),
                String::from("package_name"),
                String::from("--local"),
            ];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);

            let mut args = InstallArgs::default();
            args.packages = HashSet::from(["package_name"]);
            args.from_local_package = true;

            assert_eq!(cli_parser.commands[0], Command::Install(args));
        }

        {
            let args = vec![
                String::from("-i"),
                String::from("package_name"),
                String::from("--local"),
            ];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);

            let mut args = InstallArgs::default();
            args.packages = HashSet::from(["package_name"]);
            args.from_local_package = true;

            assert!(cli_parser.commands.contains(&Command::Install(args)));
        }

        {
            let args = vec![
                String::from("--install"),
                String::from("package_name"),
                String::from("package_name2"),
                String::from("package_name3"),
            ];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);

            let mut args = InstallArgs::default();
            args.packages = HashSet::from(["package_name", "package_name2", "package_name3"]);

            assert!(cli_parser.commands.contains(&Command::Install(args)));
        }
    }

    #[test]
    fn test_parse_update() {
        {
            let args = vec![String::from("--update"), String::from("package_name")];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);
            assert!(cli_parser
                .commands
                .contains(&Command::Update(Some("package_name"), vec![])));
        }

        {
            let args = vec![
                String::from("--update"),
                String::from("package_name"),
                String::from("--local"),
                String::from("./path/to/package_name.lod"),
            ];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);
            assert!(cli_parser.commands.contains(&Command::Update(
                Some("package_name"),
                vec![UpdateSubcommand::Local("./path/to/package_name.lod")]
            )));
        }
    }

    #[test]
    fn test_parse_delete() {
        {
            let args = vec![String::from("--delete"), String::from("package_name")];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);

            let mut args = DeleteArgs::default();
            args.packages = HashSet::from(["package_name"]);

            assert!(cli_parser.commands.contains(&Command::Delete(args)));
        }

        {
            let args = vec![
                String::from("--delete"),
                String::from("package_name"),
                String::from("package_name2"),
                String::from("package_name3"),
            ];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);

            let mut args = DeleteArgs::default();
            args.packages = HashSet::from(["package_name", "package_name2", "package_name3"]);

            assert!(cli_parser.commands.contains(&Command::Delete(args)));
        }
    }

    #[test]
    fn test_parse_module_with_subcommands() {
        {
            let args = vec![
                String::from("--module"),
                String::from("--add"),
                String::from("arg1"),
                String::from("arg2"),
                String::from("arg3"),
            ];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);
            let expected_command =
                Command::Module(ModuleSubcommand::Add(vec!["arg1", "arg2", "arg3"]));
            assert!(cli_parser.commands.contains(&expected_command));
        }

        {
            let args = vec![
                String::from("--module"),
                String::from("--delete"),
                String::from("arg1"),
                String::from("arg2"),
                String::from("arg3"),
            ];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);
            let expected_command =
                Command::Module(ModuleSubcommand::Delete(vec!["arg1", "arg2", "arg3"]));
            assert!(cli_parser.commands.contains(&expected_command));
        }

        {
            let args = vec![String::from("--module"), String::from("--list")];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);
            let expected_command = Command::Module(ModuleSubcommand::List);
            assert!(cli_parser.commands.contains(&expected_command));
        }
    }
    #[test]

    fn test_parse_repository_with_subcommands() {
        {
            let args = vec![
                String::from("--repository"),
                String::from("--add"),
                String::from("repository-name"),
                String::from("http://example.address"),
            ];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);
            let expected_command = Command::Repository(RepositorySubcommand::Add(vec![
                "repository-name",
                "http://example.address",
            ]));
            assert!(cli_parser.commands.contains(&expected_command));
        }

        {
            let args = vec![
                String::from("--repository"),
                String::from("--delete"),
                String::from("repository-name1"),
                String::from("repository-name2"),
                String::from("repository-name3"),
            ];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);
            let expected_command = Command::Repository(RepositorySubcommand::Delete(vec![
                "repository-name1",
                "repository-name2",
                "repository-name3",
            ]));
            assert!(cli_parser.commands.contains(&expected_command));
        }

        {
            let args = vec![String::from("--repository"), String::from("--list")];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);
            let expected_command = Command::Repository(RepositorySubcommand::List);
            assert!(cli_parser.commands.contains(&expected_command));
        }
    }

    #[test]
    fn test_parse_invalid_commands() {
        let args = vec![String::from("--bla-bla")];
        let cli_parser = CliParser::parse_args(&args);
        assert!(cli_parser.commands.is_empty());
    }
}
