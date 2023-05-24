#[derive(Debug, PartialEq)]
pub enum KindSubcommand<'a> {
    Add(Vec<&'a str>),
    Delete(Vec<&'a str>),
    None,
}

impl<'a> KindSubcommand<'a> {
    pub(crate) fn parse(iter: &mut dyn Iterator<Item = &'a String>) -> Self {
        if let Some(arg) = iter.next() {
            match arg.as_str() {
                "--add" | "-a" => {
                    let arguments: Vec<&str> = iter
                        .take_while(|&arg| !arg.starts_with('-'))
                        .map(|arg| arg.as_str())
                        .collect();
                    Self::Add(arguments)
                }
                "--delete" | "-d" => {
                    let arguments: Vec<&str> = iter
                        .take_while(|&arg| !arg.starts_with('-'))
                        .map(|arg| arg.as_str())
                        .collect();
                    Self::Delete(arguments)
                }
                _ => Self::None,
            }
        } else {
            Self::None
        }
    }
}
