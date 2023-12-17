#[derive(Debug, Default, PartialEq)]
pub struct CheckPathArgs<'a> {
    pub path: &'a str,
    pub print_help: bool,
}

impl<'a> CheckPathArgs<'a> {
    pub(crate) fn parse(iter: &mut dyn Iterator<Item = &'a String>) -> Self {
        let mut args = CheckPathArgs::default();

        for arg in iter {
            match arg.as_str() {
                "--help" | "-h" => {
                    args.print_help = true;
                }
                _ => {
                    args.path = arg;
                }
            }
        }

        if args.path.is_empty() {
            args.print_help = true;
        }

        args
    }

    pub(crate) fn help() -> &'static str {
        "Usage: lpm --check-path <target-path>

Options:
    -h, --help                                                Print help
"
    }
}
