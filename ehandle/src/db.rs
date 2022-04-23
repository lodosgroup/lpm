use crate::RuntimeError;
use min_sqlite3_sys::prelude::MinSqliteWrapperError;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum MigrationErrorKind {
    VersionCouldNotSet(Option<String>),
    SqliteWrapperError(Option<String>),
}

#[derive(Debug)]
pub struct MigrationError {
    pub kind: MigrationErrorKind,
    pub reason: String,
}

impl MigrationErrorKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::VersionCouldNotSet(_) => "VersionCouldNotSet",
            Self::SqliteWrapperError(_) => "SqliteWrapperError",
        }
    }

    pub fn throw(&self) -> MigrationError {
        match self {
            Self::VersionCouldNotSet(ref err) => MigrationError {
                kind: self.clone(),
                reason: err
                    .as_ref()
                    .unwrap_or(&String::from("Migration version could not set."))
                    .to_owned(),
            },
            Self::SqliteWrapperError(ref err) => MigrationError {
                kind: self.clone(),
                reason: err
                    .as_ref()
                    .unwrap_or(&String::from(
                        "An error has been occur from Sqlite wrapper library.",
                    ))
                    .to_owned(),
            },
        }
    }
}

impl From<MigrationError> for RuntimeError {
    fn from(error: MigrationError) -> Self {
        RuntimeError {
            kind: error.kind.as_str().to_string(),
            reason: error.reason,
        }
    }
}

impl From<MinSqliteWrapperError<'_>> for RuntimeError {
    fn from(error: MinSqliteWrapperError) -> Self {
        RuntimeError {
            kind: error.kind.to_string(),
            reason: error.reason,
        }
    }
}

impl From<MinSqliteWrapperError<'_>> for MigrationError {
    fn from(error: MinSqliteWrapperError) -> Self {
        MigrationErrorKind::SqliteWrapperError(Some(error.reason)).throw()
    }
}
