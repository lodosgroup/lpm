use std::io;

#[derive(Debug)]
pub struct RuntimeError {
    pub kind: String,
    pub reason: String,
}

impl From<Box<dyn std::error::Error>> for RuntimeError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        RuntimeError {
            kind: String::from("TODO"),
            reason: error.to_string(),
        }
    }
}

impl From<io::Error> for RuntimeError {
    fn from(error: io::Error) -> Self {
        RuntimeError {
            kind: String::from("io"),
            reason: error.to_string(),
        }
    }
}

