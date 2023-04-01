#[cfg(feature = "sdk")]
use crate::ResultCode;
use crate::{lpm::LpmError, ErrorCommons, MainError};

use std::ffi::NulError;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ModuleErrorKind {
    DynamicLibraryNotFound(String),
    EntrypointFunctionNotFound,
    Internal(String),
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

impl From<NulError> for LpmError<ModuleError> {
    #[track_caller]
    fn from(error: NulError) -> Self {
        ModuleErrorKind::Internal(error.to_string()).to_lpm_err()
    }
}
