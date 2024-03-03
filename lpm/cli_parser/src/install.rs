use std::collections::HashSet;

use common::some_or_error;

#[derive(Debug, PartialEq)]
pub struct InstallArgs<'a> {
    pub packages: HashSet<&'a str>,
    pub destdir: &'a str,
    pub from_local_package: bool,
    pub print_help: bool,
    // TODO:
    // install_temporary: bool,
    // repository: Option<String>,
    // workspace: Option<String>,
}

impl<'a> Default for InstallArgs<'a> {
    fn default() -> Self {
        InstallArgs {
            packages: HashSet::default(),
            destdir: "/",
            from_local_package: false,
            print_help: false,
        }
    }
}

impl<'a> InstallArgs<'a> {
    pub(crate) fn parse(iter: &mut dyn Iterator<Item = &'a String>) -> Self {
        let mut args = InstallArgs::default();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--destdir" | "-d" => {
                    args.destdir = some_or_error!(iter.next(), "Value for '--destdir' is missing. Check 'lpm --install --help' for more information.");
                }
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
    -d, --destdir         <Target path>                       Installs package to specified path, similar to GNU DESTDIR (https://www.gnu.org/prep/standards/html_node/DESTDIR.html).
    -h, --help                                                Print help

Flags:
    -l, --local                                               Activate installation from local *.lod file
    -y, --yes                                                 Preaccept the confirmation prompts
"
    }
}
