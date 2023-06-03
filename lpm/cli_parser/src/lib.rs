#![allow(dead_code)]

pub use install::InstallSubcommand;
pub use kind::KindSubcommand;
pub use module::ModuleSubcommand;
pub use repository::RepositorySubcommand;

mod install;
mod kind;
mod module;
mod repository;

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Install(&'a str, InstallSubcommand),
    Update(&'a str, Option<&'a str>),
    Delete(&'a str),
    Kind(KindSubcommand<'a>),
    Module(ModuleSubcommand<'a>),
    Repository(RepositorySubcommand<'a>),
    Configure,
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
                    if let Some(value) = iter.next() {
                        let value2 = iter.next();
                        return Command::Update(value, value2.map(|t| t.as_str()));
                    }
                }
                "--delete" | "-d" => {
                    if let Some(value) = iter.next() {
                        return Command::Delete(value);
                    }
                }
                "--kind" | "-k" => {
                    return Command::Kind(KindSubcommand::parse(&mut iter));
                }
                "--module" | "-m" => {
                    return Command::Module(ModuleSubcommand::parse(&mut iter));
                }
                "--repository" | "-r" => {
                    return Command::Repository(RepositorySubcommand::parse(&mut iter));
                }
                "--configure" | "-c" => {
                    return Command::Configure;
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
            assert_eq!(command, Command::Update("package_name", None));
        }

        {
            let args = vec![
                String::from("--update"),
                String::from("package_name"),
                String::from("./path/to/package_name.lod"),
            ];
            let command = Command::parse_args(&args);
            assert_eq!(
                command,
                Command::Update("package_name", Some("./path/to/package_name.lod"))
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
    fn test_parse_kind_with_subcommands() {
        {
            let args = vec![
                String::from("--kind"),
                String::from("--add"),
                String::from("arg1"),
                String::from("arg2"),
                String::from("arg3"),
            ];
            let command = Command::parse_args(&args);
            let expected_command = Command::Kind(KindSubcommand::Add(vec!["arg1", "arg2", "arg3"]));
            assert_eq!(command, expected_command);
        }

        {
            let args = vec![
                String::from("--kind"),
                String::from("--delete"),
                String::from("arg1"),
                String::from("arg2"),
                String::from("arg3"),
            ];
            let command = Command::parse_args(&args);
            let expected_command =
                Command::Kind(KindSubcommand::Delete(vec!["arg1", "arg2", "arg3"]));
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

        {
            let args = vec![String::from("--kind"), String::from("--bla-bla")];
            let command = Command::parse_args(&args);
            let expected_command = Command::Kind(KindSubcommand::None);
            assert_eq!(command, expected_command);
        }
    }
}
