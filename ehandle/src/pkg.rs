use crate::{lpm::LpmError, ErrorCommons, MainError};
use min_sqlite3_sys::prelude::MinSqliteWrapperError;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum PackageErrorKind {
    InvalidPackageFiles,
    UnsupportedPackageArchitecture(String),
    UnsupportedChecksumAlgorithm(String),
    InstallationFailed(String),
    DeletionFailed(String),
    AlreadyInstalled(String),
    DoesNotExists(String),
    UnrecognizedRepository(String),
    DbOperationFailed(String),
}

impl ErrorCommons<PackageError> for PackageErrorKind {
    fn as_str(&self) -> &str {
        match self {
            Self::InvalidPackageFiles => "InvalidPackageFiles",
            Self::UnsupportedChecksumAlgorithm(_) => "UnsupportedChecksumAlgorithm",
            Self::UnsupportedPackageArchitecture(_) => "UnsupportedPackageArchitecture",
            Self::InstallationFailed(_) => "InstallationFailed",
            Self::DeletionFailed(_) => "DeletionFailed",
            Self::AlreadyInstalled(_) => "AlreadyInstalled",
            Self::DoesNotExists(_) => "DoesNotExists",
            Self::UnrecognizedRepository(_) => "UnrecognizedRepository",
            Self::DbOperationFailed(_) => "DbOperationFailed",
        }
    }

    fn throw(&self) -> PackageError {
        match self {
            Self::InvalidPackageFiles => PackageError {
                kind: self.clone(),
                reason: String::from(
                    "According to the checksum file, the package files are not valid.",
                ),
            },
            Self::UnsupportedChecksumAlgorithm(ref algorithm) => PackageError {
                kind: self.clone(),
                reason: format!("Checksum algorithm '{}' is not supported.", algorithm),
            },
            Self::UnsupportedPackageArchitecture(ref arch) => PackageError {
                kind: self.clone(),
                reason: format!(
                    "The package you are trying to install is built for '{}' architecture.",
                    arch
                ),
            },
            Self::InstallationFailed(ref package) => PackageError {
                kind: self.clone(),
                reason: format!(
                    "Installation process of '{}' package has been failed.",
                    package
                ),
            },
            Self::DeletionFailed(ref package) => PackageError {
                kind: self.clone(),
                reason: format!("Deletion process of '{}' package has been failed.", package),
            },
            Self::AlreadyInstalled(ref package) => PackageError {
                kind: self.clone(),
                reason: format!("Package '{}' already installed on your machine.", package),
            },
            Self::DoesNotExists(ref package) => PackageError {
                kind: self.clone(),
                reason: format!("Package '{}' is not installed in the system.", package),
            },
            Self::UnrecognizedRepository(ref repository) => PackageError {
                kind: self.clone(),
                reason: format!("Repository '{}' in the package you'r installing is not defined in your system.", repository)
            },
            Self::DbOperationFailed(ref error) => PackageError {
                kind: self.clone(),
                reason: error.to_string()
            },
        }
    }
}

#[derive(Debug)]
pub struct PackageError {
    kind: PackageErrorKind,
    reason: String,
}

impl From<LpmError<PackageError>> for LpmError<MainError> {
    #[track_caller]
    fn from(error: LpmError<PackageError>) -> Self {
        let e = MainError {
            kind: error.error_type.kind.as_str().to_string(),
            reason: error.error_type.reason,
        };

        LpmError::new_with_traces(e, error.chain)
    }
}

impl From<MinSqliteWrapperError<'_>> for LpmError<PackageError> {
    #[track_caller]
    fn from(error: MinSqliteWrapperError) -> Self {
        LpmError::new(PackageErrorKind::DbOperationFailed(error.reason).throw())
    }
}
