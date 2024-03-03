#[derive(Debug, PartialEq)]
pub enum UpdateSubcommand<'a> {
    Local(&'a str),
    Index,
    Db,
    Packages,
    All,
    Help,
    None,
}

impl<'a> UpdateSubcommand<'a> {
    pub(crate) fn parse(iter: &mut dyn Iterator<Item = &'a String>) -> Self {
        if let Some(arg) = iter.next() {
            match arg.as_str() {
                "--local" | "-L" => {
                    if let Some(value) = iter.next() {
                        Self::Local(value)
                    } else {
                        Self::None
                    }
                }
                "--all" | "-a" => Self::All,
                "--packages" | "-p" => Self::Packages,
                "--index" | "-i" => Self::Index,
                "--db" | "-d" => Self::Db,
                "--help" | "-h" => Self::Help,
                _ => Self::None,
            }
        } else {
            Self::None
        }
    }

    pub(crate) fn help() -> &'static str {
        "Usage: lpm --update [FLAGS] <Package Name or Path>/[OPTION]

Options:
    -a, --all                                                 Update everything(packages, repository index, db migrations)
    -p, --packages                                            Update all the installed packages
    -i, --index                                               Update repository index from remote
    -d, --db                                                  Update lpm database(by applying remote migrations)
    -l, --local         <Path of *.lod file>                  Updates from local *.lod file
    -h, --help                                                Print help

Flags:
    -y, --yes                                                 Preaccept the confirmation prompts
"
    }
}
