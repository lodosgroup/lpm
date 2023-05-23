#![allow(dead_code)]

#[derive(Debug, PartialEq)]
enum Command<'a> {
    Install(&'a str, Subcommand<'a>),
    Update(&'a str),
    Delete(&'a str),
    Kind(Subcommand<'a>),
    Module(Subcommand<'a>),
    Configure,
    Help,
}

#[derive(Debug, PartialEq)]
enum Subcommand<'a> {
    Local,
    Repository(&'a str),
    Add(Vec<&'a str>),
    Delete(Vec<&'a str>),
    Help,
}

fn parse_args(args: &[String]) -> Command<'_> {
    let mut iter = args.iter().peekable();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--install" | "-i" => {
                if let Some(value) = iter.next() {
                    let subcommand = parse_subcommand(&mut iter);
                    return Command::Install(value, subcommand);
                }
            }
            "--update" | "-u" => {
                if let Some(value) = iter.next() {
                    return Command::Update(value);
                }
            }
            "--delete" | "-d" => {
                if let Some(value) = iter.next() {
                    return Command::Delete(value);
                }
            }
            "--kind" | "-k" => {
                let subcommand = parse_subcommand(&mut iter);
                return Command::Kind(subcommand);
            }
            "--module" | "-m" => {
                let subcommand = parse_subcommand(&mut iter);
                return Command::Module(subcommand);
            }
            "--configure" | "-c" => {
                return Command::Configure;
            }
            _ => {}
        }
    }

    Command::Help
}

fn parse_subcommand<'a>(iter: &mut dyn Iterator<Item = &'a String>) -> Subcommand<'a> {
    if let Some(arg) = iter.next() {
        match arg.as_str() {
            "--local" | "-l" => Subcommand::Local,
            "--repository" | "-r" => {
                if let Some(value) = iter.next() {
                    return Subcommand::Repository(value);
                }
                Subcommand::Help
            }
            "--add" | "-a" => {
                let arguments: Vec<&str> = iter
                    .take_while(|&arg| !arg.starts_with('-'))
                    .map(|arg| arg.as_str())
                    .collect();
                return Subcommand::Add(arguments);
            }
            "--delete" | "-d" => {
                let arguments: Vec<&str> = iter
                    .take_while(|&arg| !arg.starts_with('-'))
                    .map(|arg| arg.as_str())
                    .collect();
                return Subcommand::Delete(arguments);
            }
            _ => Subcommand::Help,
        }
    } else {
        Subcommand::Help
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
                String::from("--local"),
            ];
            let command = parse_args(&args);
            assert_eq!(command, Command::Install("package_name", Subcommand::Local));
        }

        {
            let args = vec![
                String::from("--install"),
                String::from("package_name"),
                String::from("--repository"),
                String::from("https://repo.test.com"),
            ];
            let command = parse_args(&args);
            assert_eq!(
                command,
                Command::Install(
                    "package_name",
                    Subcommand::Repository("https://repo.test.com")
                )
            );
        }
    }

    #[test]
    fn test_parse_update() {
        let args = vec![String::from("--update"), String::from("package_name")];
        let command = parse_args(&args);
        assert_eq!(command, Command::Update("package_name"));
    }

    #[test]
    fn test_parse_delete() {
        let args = vec![String::from("--delete"), String::from("package_name")];
        let command = parse_args(&args);
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
            let command = parse_args(&args);
            let expected_command = Command::Module(Subcommand::Add(vec!["arg1", "arg2", "arg3"]));
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
            let command = parse_args(&args);
            let expected_command =
                Command::Module(Subcommand::Delete(vec!["arg1", "arg2", "arg3"]));
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
            let command = parse_args(&args);
            let expected_command = Command::Kind(Subcommand::Add(vec!["arg1", "arg2", "arg3"]));
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
            let command = parse_args(&args);
            let expected_command = Command::Kind(Subcommand::Delete(vec!["arg1", "arg2", "arg3"]));
            assert_eq!(command, expected_command);
        }
    }

    #[test]
    fn test_parse_invalid_commands() {
        {
            let args = vec![String::from("--bla-bla")];
            let command = parse_args(&args);
            assert_eq!(command, Command::Help);
        }

        {
            let args = vec![
                String::from("--install"),
                String::from("package_name"),
                String::from("--repository"),
            ];
            let command = parse_args(&args);
            assert_eq!(command, Command::Install("package_name", Subcommand::Help));
        }

        {
            let args = vec![String::from("--kind"), String::from("--bla-bla")];
            let command = parse_args(&args);
            let expected_command = Command::Kind(Subcommand::Help);
            assert_eq!(command, expected_command);
        }
    }
}
