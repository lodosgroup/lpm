#[derive(Debug, PartialEq)]
pub enum RepositorySubcommand<'a> {
    Add(Vec<&'a str>),
    Delete(Vec<&'a str>),
    List,
    None,
}

impl<'a> RepositorySubcommand<'a> {
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
                "--list" | "-l" => Self::List,
                _ => Self::None,
            }
        } else {
            Self::None
        }
    }
}
