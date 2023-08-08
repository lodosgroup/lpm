pub use install::InstallSubcommand;
pub use module::ModuleSubcommand;
pub use repository::RepositorySubcommand;
pub use update::UpdateSubcommand;

mod install;
mod module;
mod repository;
mod update;

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Install(&'a str, InstallSubcommand),
    Update(Option<&'a str>, Vec<UpdateSubcommand<'a>>),
    Delete(&'a str),
    Module(ModuleSubcommand<'a>),
    Repository(RepositorySubcommand<'a>),
    Version,
}

#[derive(Default)]
pub struct CliParser<'a> {
    pub commands: Vec<Command<'a>>,
    pub force_yes: bool,
}

impl CliParser<'_> {
    pub fn parse_args(args: &[String]) -> CliParser<'_> {
        let mut iter = args.iter().peekable();

        let mut cli_parser = CliParser::default();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--install" | "-i" => {
                    if let Some(value) = iter.next() {
                        cli_parser
                            .commands
                            .push(Command::Install(value, InstallSubcommand::parse(&mut iter)));
                    }
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
                    if let Some(value) = iter.next() {
                        cli_parser.commands.push(Command::Delete(value));
                    }
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
                "--yes" | "-y" => {
                    cli_parser.force_yes = true;
                }
                "--version" | "-v" => {
                    cli_parser.commands.push(Command::Version);
                }
                _ => {}
            }
        }

        cli_parser
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_install_with_subcommands() {
        {
            let args = vec![
                String::from("--install"),
                String::from("package_name"),
                String::from("-L"),
            ];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);
            assert!(cli_parser
                .commands
                .contains(&Command::Install("package_name", InstallSubcommand::Local)));
        }

        {
            let args = vec![
                String::from("--install"),
                String::from("package_name"),
                String::from("--local"),
            ];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);
            assert!(cli_parser
                .commands
                .contains(&Command::Install("package_name", InstallSubcommand::Local)));
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
        let args = vec![String::from("--delete"), String::from("package_name")];
        let cli_parser = CliParser::parse_args(&args);
        assert_eq!(cli_parser.commands.len(), 1);
        assert!(cli_parser
            .commands
            .contains(&Command::Delete("package_name")));
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
        {
            let args = vec![String::from("--bla-bla")];
            let cli_parser = CliParser::parse_args(&args);
            assert!(cli_parser.commands.is_empty());
        }

        {
            let args = vec![
                String::from("--install"),
                String::from("package_name"),
                String::from("--repository"),
            ];
            let cli_parser = CliParser::parse_args(&args);
            assert_eq!(cli_parser.commands.len(), 1);
            assert!(cli_parser
                .commands
                .contains(&Command::Install("package_name", InstallSubcommand::None)));
        }
    }
}
