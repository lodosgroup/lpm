use crate::{lpm::LpmError, ErrorCommons, RuntimeError};
use min_sqlite3_sys::prelude::MinSqliteWrapperError;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum PackageErrorKind {
    InvalidPackageFiles(Option<String>),
    UnsupportedPackageArchitecture(Option<String>),
    UnsupportedChecksumAlgorithm(Option<String>),
    InstallationFailed(Option<String>),
    DeletionFailed(Option<String>),
    AlreadyInstalled(Option<String>),
    DoesNotExists(Option<String>),
    UnrecognizedRepository(Option<String>),
}

impl ErrorCommons<PackageError> for PackageErrorKind {
    fn as_str(&self) -> &str {
        match self {
            Self::InvalidPackageFiles(_) => "InvalidPackageFiles",
            Self::UnsupportedChecksumAlgorithm(_) => "UnsupportedChecksumAlgorithm",
            Self::UnsupportedPackageArchitecture(_) => "UnsupportedPackageArchitecture",
            Self::InstallationFailed(_) => "InstallationFailed",
            Self::DeletionFailed(_) => "DeletionFailed",
            Self::AlreadyInstalled(_) => "AlreadyInstalled",
            Self::DoesNotExists(_) => "DoesNotExists",
            Self::UnrecognizedRepository(_) => "UnrecognizedRepository",
        }
    }

    fn throw(&self) -> PackageError {
        match self {
            Self::InvalidPackageFiles(ref err) => PackageError {
                kind: self.clone(),
                reason: err
                    .as_ref()
                    .unwrap_or(&String::from(
                        "According to the checksum file, the package files are not valid.",
                    ))
                    .to_owned(),
            },
            Self::UnsupportedChecksumAlgorithm(ref err) => PackageError {
                kind: self.clone(),
                reason: err
                    .as_ref()
                    .unwrap_or(&String::from(
                        "The checksum algorithm of the package is not supported.",
                    ))
                    .to_owned(),
            },
            Self::UnsupportedPackageArchitecture(ref err) => PackageError {
                kind: self.clone(),
                reason: err
                    .as_ref()
                    .unwrap_or(&String::from(
                        "The package you are trying to install is built for different system architecture and not supported by this machine.",
                    ))
                    .to_owned(),
            },
            Self::InstallationFailed(ref err) => PackageError {
                kind: self.clone(),
                reason: err
                    .as_ref()
                    .unwrap_or(&String::from(
                        "The installation process could not be completed.",
                    ))
                    .to_owned(),
            },
            Self::DeletionFailed(ref err) => PackageError {
                kind: self.clone(),
                reason: err
                    .as_ref()
                    .unwrap_or(&String::from(
                        "The deletion process could not be completed.",
                    ))
                    .to_owned(),
            },
            Self::AlreadyInstalled(ref err) => PackageError {
                kind: self.clone(),
                reason: err
                    .as_ref()
                    .unwrap_or(&String::from(
                        "The package you are trying to install is already installed in the system.",
                    ))
                    .to_owned(),
            },
            Self::DoesNotExists(ref err) => PackageError {
                kind: self.clone(),
                reason: err
                    .as_ref()
                    .unwrap_or(&String::from(
                        "The package you are trying to reach is not installed in the system.",
                    ))
                    .to_owned(),
            },
            Self::UnrecognizedRepository(ref err) => PackageError {
                kind: self.clone(),
                reason: err
                    .as_ref()
                    .unwrap_or(&String::from(
                        "The repository specified in the package is not defined in your system.",
                    ))
                    .to_owned(),
            },
        }
    }
}

#[derive(Debug)]
pub struct PackageError {
    pub kind: PackageErrorKind,
    pub reason: String,
}

impl From<LpmError<PackageError>> for LpmError<RuntimeError> {
    #[track_caller]
    fn from(error: LpmError<PackageError>) -> Self {
        let e = RuntimeError {
            kind: error.error_type.kind.as_str().to_string(),
            reason: error.error_type.reason,
        };

        LpmError::new_with_traces(e, error.chain)
    }
}

impl From<MinSqliteWrapperError<'_>> for LpmError<PackageError> {
    #[track_caller]
    fn from(error: MinSqliteWrapperError) -> Self {
        LpmError::new(PackageErrorKind::InstallationFailed(Some(error.reason)).throw())
    }
}
