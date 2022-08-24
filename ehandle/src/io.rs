use crate::{lpm::LpmError, MainError};
use std::io;

impl From<io::Error> for LpmError<io::Error> {
    #[track_caller]
    fn from(error: io::Error) -> Self {
        LpmError::new(error)
    }
}

impl From<io::Error> for LpmError<MainError> {
    #[track_caller]
    fn from(error: io::Error) -> Self {
        // Reformat kind value
        // e.g 'permission denied' value will be converted
        // into 'PermissionDenied'
        let words = error.kind().to_string();
        let words = words.split(' ');
        let mut kind = String::new();
        for word in words {
            let s = word[0..1].to_uppercase() + &word[1..];
            kind = format!("{}{}", kind, s);
        }

        LpmError::new(MainError {
            kind,
            reason: error.to_string(),
        })
    }
}

impl From<LpmError<io::Error>> for LpmError<MainError> {
    #[track_caller]
    fn from(error: LpmError<io::Error>) -> Self {
        let e = MainError {
            kind: error.error_type.kind().to_string(),
            reason: error.error_type.to_string(),
        };
        LpmError::new_with_traces(e, error.chain)
    }
}
