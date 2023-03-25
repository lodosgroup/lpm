use crate::{lpm::LpmError, ErrorCommons, MainError};

use std::ffi::NulError;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum PluginErrorKind {
    DynamicLibraryNotFound(String),
    EntrypointFunctionNotFound,
    Internal(String),
}

#[derive(Debug)]
pub struct PluginError {
    kind: String,
    reason: String,
}

impl ErrorCommons<PluginError> for PluginErrorKind {
    fn as_str(&self) -> &str {
        match self {
            PluginErrorKind::DynamicLibraryNotFound(_) => "DynamicLibraryNotFound",
            PluginErrorKind::EntrypointFunctionNotFound => "EntrypointFunctionNotFound",
            PluginErrorKind::Internal(_) => "Internal",
        }
    }

    fn to_err(&self) -> PluginError {
        match self {
            PluginErrorKind::DynamicLibraryNotFound(dylib_path) => PluginError {
                kind: self.as_str().to_owned(),
                reason: format!("Dynamic library is not found at '{}'.", dylib_path),
            },
            PluginErrorKind::EntrypointFunctionNotFound => PluginError {
                kind: self.as_str().to_owned(),
                reason: String::from(
                    "'lpm_entrypoint' function is not found in the dynamic library.",
                ),
            },
            PluginErrorKind::Internal(reason) => PluginError {
                kind: self.as_str().to_owned(),
                reason: reason.to_owned(),
            },
        }
    }

    fn to_lpm_err(&self) -> crate::lpm::LpmError<PluginError> {
        LpmError::new(self.to_err())
    }
}

impl From<LpmError<PluginError>> for LpmError<MainError> {
    #[track_caller]
    fn from(error: LpmError<PluginError>) -> Self {
        let e = MainError {
            kind: error.error_type.kind.as_str().to_string(),
            reason: error.error_type.reason,
        };

        LpmError::new_with_traces(e, error.chain)
    }
}

impl From<NulError> for LpmError<PluginError> {
    #[track_caller]
    fn from(error: NulError) -> Self {
        PluginErrorKind::Internal(error.to_string()).to_lpm_err()
    }
}
