use std::collections::HashSet;

#[derive(Debug, Default, PartialEq)]
pub struct DeleteArgs<'a> {
    pub packages: HashSet<&'a str>,
    pub print_help: bool,
}

impl<'a> DeleteArgs<'a> {
    pub(crate) fn parse(iter: &mut dyn Iterator<Item = &'a String>) -> Self {
        let mut args = DeleteArgs::default();

        for arg in iter {
            match arg.as_str() {
                "--help" | "-h" => {
                    args.print_help = true;
                }
                _ => {
                    args.packages.insert(arg);
                }
            }
        }

        if args.packages.is_empty() {
            args.print_help = true;
        }

        args
    }

    pub(crate) fn help() -> &'static str {
        "Usage: lpm --delete [FLAGS] <List of package names>/[OPTION]

Options:
    -h, --help                                                Print help

Flags:
    -y, --yes                                                 Preaccept the confirmation prompts
"
    }
}
