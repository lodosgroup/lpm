#[cfg(feature = "sdk")]
use crate::ResultCode;
use crate::{lpm::LpmError, ErrorCommons, MainError};

use min_sqlite3_sys::prelude::MinSqliteWrapperError;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum PackageErrorKind {
    InvalidPackageFiles,
    UnsupportedPackageArchitecture(String),
    UnsupportedChecksumAlgorithm(String),
    InstallationFailed(String),
    UnsupportedStandard(String, String),
    DeletionFailed(String),
    AlreadyInstalled(String),
    DoesNotExists(String),
    UnrecognizedRepository(String),
    DbOperationFailed(String),
    FailedExecutingStage1Script { script_name: String, output: String },
    InvalidPackageName(String),
}

impl ErrorCommons for PackageErrorKind {
    type Error = PackageError;

    fn as_str(&self) -> &str {
        match self {
            Self::InvalidPackageFiles => "InvalidPackageFiles",
            Self::UnsupportedChecksumAlgorithm(_) => "UnsupportedChecksumAlgorithm",
            Self::UnsupportedPackageArchitecture(_) => "UnsupportedPackageArchitecture",
            Self::InstallationFailed(_) => "InstallationFailed",
            Self::UnsupportedStandard(..) => "ExtractionFailed",
            Self::DeletionFailed(_) => "DeletionFailed",
            Self::AlreadyInstalled(_) => "AlreadyInstalled",
            Self::DoesNotExists(_) => "DoesNotExists",
            Self::UnrecognizedRepository(_) => "UnrecognizedRepository",
            Self::DbOperationFailed(_) => "DbOperationFailed",
            Self::FailedExecutingStage1Script { .. } => "FailedExecutingStage1Script",
            Self::InvalidPackageName(_) => "InvalidPackageName",
        }
    }

    fn to_err(&self) -> Self::Error {
        match self {
            Self::InvalidPackageFiles => Self::Error {
                kind: self.as_str().to_owned(),
                reason: String::from(
                    "According to the checksum file, the package files are not valid.",
                ),
            },
            Self::UnsupportedChecksumAlgorithm(ref algorithm) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!("Checksum algorithm '{}' is not supported.", algorithm),
            },
            Self::UnsupportedPackageArchitecture(ref arch) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!(
                    "The package you are trying to install is built for '{}' architecture.",
                    arch
                ),
            },
            Self::InstallationFailed(ref package) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!(
                    "Installation process of '{}' package has been failed.",
                    package
                ),
            },
            Self::UnsupportedStandard(ref package, ref error) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!(
                    "Extraction process of '{}' package has been failed. Error: {}",
                    package, error
                ),
            },
            Self::DeletionFailed(ref package) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!("Deletion process of '{}' package has been failed.", package),
            },
            Self::AlreadyInstalled(ref package) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!("Package '{}' already installed on your machine.", package),
            },
            Self::DoesNotExists(ref package) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!("Package '{}' is not installed in the system.", package),
            },
            Self::UnrecognizedRepository(ref repository) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!("Repository '{}' in the package you'r installing is not defined in your system.", repository)
            },
            Self::DbOperationFailed(ref error) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: error.to_string()
            },
            Self::FailedExecutingStage1Script{ script_name, output } => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!("Stage1 script '{}' failed. Output: {}", script_name, output)
            },
            Self::InvalidPackageName(ref pkg_name) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!("'{pkg_name}' is not a valid package name.")
            },
        }
    }

    #[inline]
    #[cfg(feature = "sdk")]
    fn to_lpm_err(&self) -> LpmError<Self::Error> {
        LpmError::new(self.to_err(), self.to_result_code())
    }

    #[inline]
    #[cfg(not(feature = "sdk"))]
    fn to_lpm_err(&self) -> LpmError<Self::Error> {
        LpmError::new(self.to_err())
    }

    #[cfg(feature = "sdk")]
    fn to_result_code(&self) -> ResultCode {
        match self {
            PackageErrorKind::InvalidPackageFiles => ResultCode::PackageError_InvalidPackageFiles,
            PackageErrorKind::UnsupportedPackageArchitecture(_) => {
                ResultCode::PackageError_UnsupportedPackageArchitecture
            }
            PackageErrorKind::UnsupportedChecksumAlgorithm(_) => {
                ResultCode::PackageError_UnsupportedChecksumAlgorithm
            }
            PackageErrorKind::InstallationFailed(_) => ResultCode::PackageError_InstallationFailed,
            PackageErrorKind::UnsupportedStandard(_, _) => {
                ResultCode::PackageError_UnsupportedStandard
            }
            PackageErrorKind::DeletionFailed(_) => ResultCode::PackageError_DeletionFailed,
            PackageErrorKind::AlreadyInstalled(_) => ResultCode::PackageError_AlreadyInstalled,
            PackageErrorKind::DoesNotExists(_) => ResultCode::PackageError_DoesNotExists,
            PackageErrorKind::UnrecognizedRepository(_) => {
                ResultCode::PackageError_UnrecognizedRepository
            }
            PackageErrorKind::DbOperationFailed(_) => ResultCode::PackageError_DbOperationFailed,
            PackageErrorKind::FailedExecutingStage1Script { .. } => {
                ResultCode::PackageError_FailedExecutingStage1Script
            }
            PackageErrorKind::InvalidPackageName(_) => ResultCode::PackageError_InvalidPackageName,
        }
    }
}

#[derive(Debug)]
pub struct PackageError {
    kind: String,
    reason: String,
}

impl From<LpmError<PackageError>> for LpmError<MainError> {
    #[track_caller]
    #[cfg(feature = "sdk")]
    fn from(error: LpmError<PackageError>) -> Self {
        let e = MainError {
            kind: error.error_type.kind,
            reason: error.error_type.reason,
        };

        let result_tag = "PackageError";
        let result_code = ResultCode::from_str(&format!("{}_{}", result_tag, &e.kind));
        LpmError::new_with_traces(e, result_code, error.chain)
    }

    #[track_caller]
    #[cfg(not(feature = "sdk"))]
    fn from(error: LpmError<PackageError>) -> Self {
        let e = MainError {
            kind: error.error_type.kind,
            reason: error.error_type.reason,
        };

        LpmError::new_with_traces(e, error.chain)
    }
}

impl From<MinSqliteWrapperError<'_>> for LpmError<PackageError> {
    #[track_caller]
    fn from(error: MinSqliteWrapperError) -> Self {
        PackageErrorKind::DbOperationFailed(error.reason).to_lpm_err()
    }
}
