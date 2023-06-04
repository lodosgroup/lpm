#[derive(Debug, PartialEq)]
pub enum UpdateSubcommand<'a> {
    Local(&'a str),
    Index,
    Db,
    Packages,
    All,
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
                _ => Self::None,
            }
        } else {
            Self::None
        }
    }
}
