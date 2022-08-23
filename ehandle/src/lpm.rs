use std::panic::Location;

pub struct ErrorStack {
    pub file: String,
    pub column: u32,
    pub line: u32,
}

impl std::fmt::Debug for ErrorStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}:{}\"", self.file, self.line)
    }
}

impl ErrorStack {
    #[track_caller]
    fn new() -> Self {
        let caller = Location::caller();
        Self {
            file: caller.file().to_string(),
            column: caller.column(),
            line: caller.line(),
        }
    }
}

#[derive(Debug)]
pub struct LpmError<E> {
    pub error_type: E,
    pub chain: Vec<ErrorStack>,
}

impl<E> LpmError<E> {
    #[track_caller]
    pub fn new(e: E) -> Self {
        Self {
            error_type: e,
            chain: vec![ErrorStack::new()],
        }
    }

    #[track_caller]
    pub fn new_with_traces(e: E, chain: Vec<ErrorStack>) -> Self {
        let mut e = Self {
            error_type: e,
            chain,
        };
        e.chain.push(ErrorStack::new());

        e
    }
}
