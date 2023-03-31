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

impl ErrorCommons<ModuleError> for ModuleErrorKind {
    fn as_str(&self) -> &str {
        match self {
            ModuleErrorKind::DynamicLibraryNotFound(_) => "DynamicLibraryNotFound",
            ModuleErrorKind::EntrypointFunctionNotFound => "EntrypointFunctionNotFound",
            ModuleErrorKind::Internal(_) => "Internal",
        }
    }

    fn to_err(&self) -> ModuleError {
        match self {
            ModuleErrorKind::DynamicLibraryNotFound(dylib_path) => ModuleError {
                kind: self.as_str().to_owned(),
                reason: format!("Dynamic library is not found at '{}'.", dylib_path),
            },
            ModuleErrorKind::EntrypointFunctionNotFound => ModuleError {
                kind: self.as_str().to_owned(),
                reason: String::from(
                    "'lpm_entrypoint' function is not found in the dynamic library.",
                ),
            },
            ModuleErrorKind::Internal(reason) => ModuleError {
                kind: self.as_str().to_owned(),
                reason: reason.to_owned(),
            },
        }
    }

    fn to_lpm_err(&self) -> crate::lpm::LpmError<ModuleError> {
        LpmError::new(self.to_err())
    }
}

impl From<LpmError<ModuleError>> for LpmError<MainError> {
    #[track_caller]
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
