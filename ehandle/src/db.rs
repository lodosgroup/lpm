use crate::RuntimeError;
use min_sqlite3_sys::prelude::MinSqliteWrapperError;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum MigrationErrorKind {
    VersionCouldNotSet,
    SqliteWrapperError,
}

impl MigrationErrorKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::VersionCouldNotSet => "VersionCouldNotSet",
            Self::SqliteWrapperError => "SqliteWrapperError",
        }
    }
}

#[derive(Debug)]
pub struct MigrationError {
    pub kind: MigrationErrorKind,
    pub reason: String,
}

impl MigrationError {
    pub fn new(kind: MigrationErrorKind) -> Self {
        match kind {
            MigrationErrorKind::VersionCouldNotSet => MigrationError {
                kind,
                reason: "Migration version could not set.".to_string(),
            },
            MigrationErrorKind::SqliteWrapperError => MigrationError {
                kind,
                reason: "An error has been occur from Sqlite wrapper library.".to_string(),
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
        MigrationError {
            kind: MigrationErrorKind::SqliteWrapperError,
            reason: error.reason,
        }
    }
}
