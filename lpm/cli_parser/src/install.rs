#[derive(Debug, PartialEq)]
pub enum InstallSubcommand {
    Local,
    Help,
    None,
}

impl InstallSubcommand {
    pub(crate) fn parse(iter: &mut dyn Iterator<Item = &String>) -> Self {
        if let Some(arg) = iter.next() {
            match arg.as_str() {
                "--help" | "-h" => Self::Help,
                "--local" | "-L" => Self::Local,
                _ => Self::None,
            }
        } else {
            Self::None
        }
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
