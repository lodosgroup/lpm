use std::collections::HashSet;

#[derive(Debug, Default, PartialEq)]
pub struct InstallArgs<'a> {
    pub packages: HashSet<&'a str>,
    pub from_local_package: bool,
    pub print_help: bool,
    // TODO:
    // install_temporary: bool,
    // repository: Option<String>,
    // workspace: Option<String>,
}

impl<'a> InstallArgs<'a> {
    pub(crate) fn parse(iter: &mut dyn Iterator<Item = &'a String>) -> Self {
        let mut args = InstallArgs::default();

        for arg in iter {
            match arg.as_str() {
                "--local" | "-L" => {
                    args.from_local_package = true;
                }
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
        "Usage: lpm --install [FLAGS] <List of package names or Path>/[OPTION]

Options:
    -h, --help                                                Print help

Flags:
    -l, --local                                               Activate installation from local *.lod file
    -y, --yes                                                 Preaccept the confirmation prompts
"
    }
}
