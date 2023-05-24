#[derive(Debug, PartialEq)]
pub enum InstallSubcommand {
    None,
    Local,
}

impl InstallSubcommand {
    pub(crate) fn parse(iter: &mut dyn Iterator<Item = &String>) -> Self {
        if let Some(arg) = iter.next() {
            match arg.as_str() {
                "--local" | "-L" => Self::Local,
                _ => Self::None,
            }
        } else {
            Self::None
        }
    }
}
