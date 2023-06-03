use crate::db::SqlError;
#[cfg(feature = "sdk")]
use crate::ResultCode;
use crate::{lpm::LpmError, ErrorCommons, MainError};

use min_sqlite3_sys::prelude::MinSqliteWrapperError;
use std::ffi::NulError;
use std::io;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum RepositoryErrorKind {
    RepositoryNotFound(String),
    RepositoryAlreadyExists(String),
    Internal(String),
}

#[derive(Debug)]
pub struct RepositoryError {
    kind: String,
    reason: String,
}

impl ErrorCommons for RepositoryErrorKind {
    type Error = RepositoryError;

    fn as_str(&self) -> &str {
        match self {
            Self::RepositoryNotFound(_) => "RepositoryNotFound",
            Self::RepositoryAlreadyExists(_) => "RepositoryAlreadyExists",
            Self::Internal(_) => "Internal",
        }
    }

    fn to_err(&self) -> Self::Error {
        match self {
            Self::RepositoryNotFound(name) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!("Repository '{}' is not found at.", name),
            },
            Self::RepositoryAlreadyExists(name) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!("Repository '{}' already exists in your system.", name),
            },
            Self::Internal(reason) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: reason.to_owned(),
            },
        }
    }

    #[cfg(feature = "sdk")]
    fn to_lpm_err(&self) -> crate::lpm::LpmError<Self::Error> {
        LpmError::new(self.to_err(), self.to_result_code())
    }

    #[cfg(not(feature = "sdk"))]
    fn to_lpm_err(&self) -> crate::lpm::LpmError<Self::Error> {
        LpmError::new(self.to_err())
    }

    #[cfg(feature = "sdk")]
    fn to_result_code(&self) -> ResultCode {
        match self {
            Self::RepositoryNotFound(_) => ResultCode::RepositoryError_RepositoryNotFound,
            Self::RepositoryAlreadyExists(_) => ResultCode::RepositoryError_RepositoryAlreadyExists,
            Self::Internal(_) => ResultCode::RepositoryError_Internal,
        }
    }
}

impl From<LpmError<RepositoryError>> for LpmError<MainError> {
    #[track_caller]
    #[cfg(feature = "sdk")]
    fn from(error: LpmError<RepositoryError>) -> Self {
        let e = MainError {
            kind: error.error_type.kind.as_str().to_string(),
            reason: error.error_type.reason,
        };

        let result_tag = "RepositoryError";
        let result_code = ResultCode::from_str(&format!("{}_{}", result_tag, &e.kind));
        LpmError::new_with_traces(e, result_code, error.chain)
    }

    #[track_caller]
    #[cfg(not(feature = "sdk"))]
    fn from(error: LpmError<RepositoryError>) -> Self {
        let e = MainError {
            kind: error.error_type.kind.as_str().to_string(),
            reason: error.error_type.reason,
        };

        LpmError::new_with_traces(e, error.chain)
    }
}

impl From<LpmError<SqlError>> for LpmError<RepositoryError> {
    #[track_caller]
    #[cfg(feature = "sdk")]
    fn from(error: LpmError<SqlError>) -> Self {
        let e = RepositoryError {
            kind: error.error_type.kind,
            reason: error.error_type.reason,
        };

        let result_tag = "SqlError";
        let result_code = ResultCode::from_str(&format!("{}_{}", result_tag, &e.kind));
        LpmError::new_with_traces(e, result_code, error.chain)
    }

    #[track_caller]
    #[cfg(not(feature = "sdk"))]
    fn from(error: LpmError<SqlError>) -> Self {
        let e = RepositoryError {
            kind: error.error_type.kind,
            reason: error.error_type.reason,
        };

        LpmError::new_with_traces(e, error.chain)
    }
}

impl From<MinSqliteWrapperError<'_>> for LpmError<RepositoryError> {
    #[track_caller]
    fn from(error: MinSqliteWrapperError) -> Self {
        LpmError::new(
            RepositoryError {
                kind: error.kind.to_owned(),
                reason: error.reason,
            },
            #[cfg(feature = "sdk")]
            ResultCode::MinSqliteWrapperError,
        )
    }
}

impl From<io::Error> for LpmError<RepositoryError> {
    #[track_caller]
    #[cfg(feature = "sdk")]
    fn from(error: io::Error) -> Self {
        LpmError::new(
            RepositoryError {
                kind: error.kind().to_string(),
                reason: error.to_string(),
            },
            error.kind().into(),
        )
    }

    #[track_caller]
    #[cfg(not(feature = "sdk"))]
    fn from(error: io::Error) -> Self {
        LpmError::new(RepositoryError {
            kind: error.kind().to_string(),
            reason: error.to_string(),
        })
    }
}

impl From<NulError> for LpmError<RepositoryError> {
    #[track_caller]
    fn from(error: NulError) -> Self {
        RepositoryErrorKind::Internal(error.to_string()).to_lpm_err()
    }
}
