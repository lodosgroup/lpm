#[cfg(feature = "sdk")]
use crate::ResultCode;
use crate::{db::SqlError, lpm::LpmError, ErrorCommons, MainError};

use min_sqlite3_sys::prelude::MinSqliteWrapperError;
use std::{ffi::NulError, io};

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ModuleErrorKind {
    DynamicLibraryNotFound(String),
    EntrypointFunctionNotFound,
    Internal(String),
    ModuleNotFound(String),
    ModuleAlreadyExists(String),
}

#[derive(Debug)]
pub struct ModuleError {
    kind: String,
    reason: String,
}

impl ErrorCommons for ModuleErrorKind {
    type Error = ModuleError;

    fn as_str(&self) -> &str {
        match self {
            ModuleErrorKind::DynamicLibraryNotFound(_) => "DynamicLibraryNotFound",
            ModuleErrorKind::EntrypointFunctionNotFound => "EntrypointFunctionNotFound",
            ModuleErrorKind::Internal(_) => "Internal",
            ModuleErrorKind::ModuleNotFound(_) => "ModuleNotFound",
            ModuleErrorKind::ModuleAlreadyExists(_) => "ModuleAlreadyExists",
        }
    }

    fn to_err(&self) -> Self::Error {
        match self {
            ModuleErrorKind::DynamicLibraryNotFound(dylib_path) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!("Dynamic library is not found at '{}'.", dylib_path),
            },
            ModuleErrorKind::EntrypointFunctionNotFound => Self::Error {
                kind: self.as_str().to_owned(),
                reason: String::from(
                    "'lpm_entrypoint' function is not found in the dynamic library.",
                ),
            },
            ModuleErrorKind::Internal(reason) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: reason.to_owned(),
            },
            ModuleErrorKind::ModuleNotFound(module_name) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!("{module_name} not found in the database."),
            },
            ModuleErrorKind::ModuleAlreadyExists(module_name) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!("{module_name} already exists in the database."),
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
            ModuleErrorKind::DynamicLibraryNotFound(_) => {
                ResultCode::ModuleError_DynamicLibraryNotFound
            }
            ModuleErrorKind::EntrypointFunctionNotFound => {
                ResultCode::ModuleError_EntrypointFunctionNotFound
            }
            ModuleErrorKind::Internal(_) => ResultCode::ModuleError_Internal,
            ModuleErrorKind::ModuleNotFound(_) => ResultCode::ModuleError_ModuleNotFound,
            ModuleErrorKind::ModuleAlreadyExists(_) => ResultCode::ModuleError_ModuleAlreadyExists,
        }
    }
}

impl From<LpmError<ModuleError>> for LpmError<MainError> {
    #[track_caller]
    #[cfg(feature = "sdk")]
    fn from(error: LpmError<ModuleError>) -> Self {
        let e = MainError {
            kind: error.error_type.kind.as_str().to_string(),
            reason: error.error_type.reason,
        };

        let result_tag = "ModuleError";
        let result_code = ResultCode::from_str(&format!("{}_{}", result_tag, &e.kind));
        LpmError::new_with_traces(e, result_code, error.chain)
    }

    #[track_caller]
    #[cfg(not(feature = "sdk"))]
    fn from(error: LpmError<ModuleError>) -> Self {
        let e = MainError {
            kind: error.error_type.kind.as_str().to_string(),
            reason: error.error_type.reason,
        };

        LpmError::new_with_traces(e, error.chain)
    }
}

impl From<LpmError<SqlError>> for LpmError<ModuleError> {
    #[track_caller]
    #[cfg(feature = "sdk")]
    fn from(error: LpmError<SqlError>) -> Self {
        let e = ModuleError {
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
        let e = ModuleError {
            kind: error.error_type.kind,
            reason: error.error_type.reason,
        };

        LpmError::new_with_traces(e, error.chain)
    }
}

impl From<MinSqliteWrapperError<'_>> for LpmError<ModuleError> {
    #[track_caller]
    fn from(error: MinSqliteWrapperError) -> Self {
        LpmError::new(
            ModuleError {
                kind: error.kind.to_owned(),
                reason: error.reason,
            },
            #[cfg(feature = "sdk")]
            ResultCode::MinSqliteWrapperError,
        )
    }
}

impl From<io::Error> for LpmError<ModuleError> {
    #[track_caller]
    #[cfg(feature = "sdk")]
    fn from(error: io::Error) -> Self {
        LpmError::new(
            ModuleError {
                kind: error.kind().to_string(),
                reason: error.to_string(),
            },
            error.kind().into(),
        )
    }

    #[track_caller]
    #[cfg(not(feature = "sdk"))]
    fn from(error: io::Error) -> Self {
        LpmError::new(ModuleError {
            kind: error.kind().to_string(),
            reason: error.to_string(),
        })
    }
}

impl From<NulError> for LpmError<ModuleError> {
    #[track_caller]
    fn from(error: NulError) -> Self {
        ModuleErrorKind::Internal(error.to_string()).to_lpm_err()
    }
}
