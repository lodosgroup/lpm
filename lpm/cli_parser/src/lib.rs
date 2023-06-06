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
    None,
}

impl Command<'_> {
    pub fn parse_args(args: &[String]) -> Command<'_> {
        let mut iter = args.iter().peekable();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--install" | "-i" => {
                    if let Some(value) = iter.next() {
                        return Command::Install(value, InstallSubcommand::parse(&mut iter));
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

                    return Command::Update(pkg_name.map(|t| t.as_str()), subcommands);
                }
                "--delete" | "-d" => {
                    if let Some(value) = iter.next() {
                        return Command::Delete(value);
                    }
                }
                "--module" | "-m" => {
                    return Command::Module(ModuleSubcommand::parse(&mut iter));
                }
                "--repository" | "-r" => {
                    return Command::Repository(RepositorySubcommand::parse(&mut iter));
                }
                _ => {}
            }
        }

        Command::None
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
            let command = Command::parse_args(&args);
            assert_eq!(
                command,
                Command::Install("package_name", InstallSubcommand::Local)
            );
        }

        {
            let args = vec![
                String::from("--install"),
                String::from("package_name"),
                String::from("--local"),
            ];
            let command = Command::parse_args(&args);
            assert_eq!(
                command,
                Command::Install("package_name", InstallSubcommand::Local)
            );
        }
    }

    #[test]
    fn test_parse_update() {
        {
            let args = vec![String::from("--update"), String::from("package_name")];
            let command = Command::parse_args(&args);
            assert_eq!(command, Command::Update(Some("package_name"), vec![]));
        }

        {
            let args = vec![
                String::from("--update"),
                String::from("package_name"),
                String::from("--local"),
                String::from("./path/to/package_name.lod"),
            ];
            let command = Command::parse_args(&args);
            assert_eq!(
                command,
                Command::Update(
                    Some("package_name"),
                    vec![UpdateSubcommand::Local("./path/to/package_name.lod")]
                )
            );
        }
    }

    #[test]
    fn test_parse_delete() {
        let args = vec![String::from("--delete"), String::from("package_name")];
        let command = Command::parse_args(&args);
        assert_eq!(command, Command::Delete("package_name"));
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
            let command = Command::parse_args(&args);
            let expected_command =
                Command::Module(ModuleSubcommand::Add(vec!["arg1", "arg2", "arg3"]));
            assert_eq!(command, expected_command);
        }

        {
            let args = vec![
                String::from("--module"),
                String::from("--delete"),
                String::from("arg1"),
                String::from("arg2"),
                String::from("arg3"),
            ];
            let command = Command::parse_args(&args);
            let expected_command =
                Command::Module(ModuleSubcommand::Delete(vec!["arg1", "arg2", "arg3"]));
            assert_eq!(command, expected_command);
        }

        {
            let args = vec![String::from("--module"), String::from("--list")];
            let command = Command::parse_args(&args);
            let expected_command = Command::Module(ModuleSubcommand::List);
            assert_eq!(command, expected_command);
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
            let command = Command::parse_args(&args);
            let expected_command = Command::Repository(RepositorySubcommand::Add(vec![
                "repository-name",
                "http://example.address",
            ]));
            assert_eq!(command, expected_command);
        }

        {
            let args = vec![
                String::from("--repository"),
                String::from("--delete"),
                String::from("repository-name1"),
                String::from("repository-name2"),
                String::from("repository-name3"),
            ];
            let command = Command::parse_args(&args);
            let expected_command = Command::Repository(RepositorySubcommand::Delete(vec![
                "repository-name1",
                "repository-name2",
                "repository-name3",
            ]));
            assert_eq!(command, expected_command);
        }

        {
            let args = vec![String::from("--repository"), String::from("--list")];
            let command = Command::parse_args(&args);
            let expected_command = Command::Repository(RepositorySubcommand::List);
            assert_eq!(command, expected_command);
        }
    }

    #[test]
    fn test_parse_invalid_commands() {
        {
            let args = vec![String::from("--bla-bla")];
            let command = Command::parse_args(&args);
            assert_eq!(command, Command::None);
        }

        {
            let args = vec![
                String::from("--install"),
                String::from("package_name"),
                String::from("--repository"),
            ];
            let command = Command::parse_args(&args);
            assert_eq!(
                command,
                Command::Install("package_name", InstallSubcommand::None)
            );
        }
    }
}
