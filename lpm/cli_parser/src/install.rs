#[derive(Debug, Default, PartialEq)]
pub struct InstallArgs {
    pub packages: Vec<String>,
    pub from_local_package: bool,
    pub print_help: bool,
    // TODO:
    // install_temporary: bool,
    // repository: Option<String>,
    // workspace: Option<String>,
}

impl InstallArgs {
    pub(crate) fn parse(iter: &mut dyn Iterator<Item = &String>) -> Self {
        let mut args = InstallArgs::default();

        for arg in iter {
            match arg.as_str() {
                "--local" | "-L" => args.from_local_package = true,
                "--help" | "-h" => args.print_help = true,
                _ => args.packages.push(arg.to_owned()),
            }
        }

        if args.packages.is_empty() {
            args.print_help = true;
        }

        args
    }

    pub(crate) fn help() -> &'static str {
        "Usage: lpm --install [FLAGS] <Package Name or Path>/[OPTION]

Options:
    -h, --help                                                Print help

Flags:
    -l, --local                                               Activate installation from local *.lod file
    -y, --yes                                                 Preaccept the confirmation prompts
"
    }
}
