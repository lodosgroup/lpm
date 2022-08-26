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
    PackageKindNotFound(String),
    MetaDirCouldNotLoad,
}

impl ErrorCommons<PackageError> for PackageErrorKind {
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
            Self::PackageKindNotFound(_) => "PackageKindNotFound",
            Self::MetaDirCouldNotLoad => "MetaDirCouldNotLoad",
        }
    }

    fn to_err(&self) -> PackageError {
        match self {
            Self::InvalidPackageFiles => PackageError {
                kind: self.as_str().to_owned(),
                reason: String::from(
                    "According to the checksum file, the package files are not valid.",
                ),
            },
            Self::UnsupportedChecksumAlgorithm(ref algorithm) => PackageError {
                kind: self.as_str().to_owned(),
                reason: format!("Checksum algorithm '{}' is not supported.", algorithm),
            },
            Self::UnsupportedPackageArchitecture(ref arch) => PackageError {
                kind: self.as_str().to_owned(),
                reason: format!(
                    "The package you are trying to install is built for '{}' architecture.",
                    arch
                ),
            },
            Self::InstallationFailed(ref package) => PackageError {
                kind: self.as_str().to_owned(),
                reason: format!(
                    "Installation process of '{}' package has been failed.",
                    package
                ),
            },
            Self::UnsupportedStandard(ref package, ref error) => PackageError {
                kind: self.as_str().to_owned(),
                reason: format!(
                    "Extraction process of '{}' package has been failed. Error: {}",
                    package, error
                ),
            },
            Self::DeletionFailed(ref package) => PackageError {
                kind: self.as_str().to_owned(),
                reason: format!("Deletion process of '{}' package has been failed.", package),
            },
            Self::AlreadyInstalled(ref package) => PackageError {
                kind: self.as_str().to_owned(),
                reason: format!("Package '{}' already installed on your machine.", package),
            },
            Self::DoesNotExists(ref package) => PackageError {
                kind: self.as_str().to_owned(),
                reason: format!("Package '{}' is not installed in the system.", package),
            },
            Self::UnrecognizedRepository(ref repository) => PackageError {
                kind: self.as_str().to_owned(),
                reason: format!("Repository '{}' in the package you'r installing is not defined in your system.", repository)
            },
            Self::DbOperationFailed(ref error) => PackageError {
                kind: self.as_str().to_owned(),
                reason: error.to_string()
            },
            Self::PackageKindNotFound(ref kind) => PackageError {
                kind: self.as_str().to_owned(),
                reason: format!("Kind '{}' does not exists in the database.", kind)
            },
            Self::MetaDirCouldNotLoad => PackageError {
                kind: self.as_str().to_owned(),
                reason: String::from("Meta directory of the package could not be loaded.")
            },
        }
    }

    #[inline]
    fn to_lpm_err(&self) -> LpmError<PackageError> {
        LpmError::new(self.to_err())
    }
}

#[derive(Debug)]
pub struct PackageError {
    kind: String,
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
        PackageErrorKind::DbOperationFailed(error.reason).to_lpm_err()
    }
}
