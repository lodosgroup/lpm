use min_sqlite3_sys::prelude::MinSqliteWrapperError;

use crate::{ErrorCommons, RuntimeError};

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum PackageErrorKind {
    InvalidPackageFiles(Option<String>),
    UnsupportedPackageArchitecture(Option<String>),
    UnsupportedChecksumAlgorithm(Option<String>),
    InstallationFailed(Option<String>),
    AlreadyInstalled(Option<String>),
}

impl ErrorCommons<PackageError> for PackageErrorKind {
    #[inline(always)]
    fn as_str(&self) -> &str {
        match self {
            Self::InvalidPackageFiles(_) => "InvalidPackageFiles",
            Self::UnsupportedChecksumAlgorithm(_) => "UnsupportedChecksumAlgorithm",
            Self::UnsupportedPackageArchitecture(_) => "UnsupportedPackageArchitecture",
            Self::InstallationFailed(_) => "InstallationFailed",
            Self::AlreadyInstalled(_) => "AlreadyInstalled",
        }
    }

    #[inline(always)]
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
            Self::AlreadyInstalled(ref err) => PackageError {
                kind: self.clone(),
                reason: err
                    .as_ref()
                    .unwrap_or(&String::from(
                        "The package you are trying to install is already installed in the system.",
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

impl From<PackageError> for RuntimeError {
    #[inline(always)]
    fn from(error: PackageError) -> Self {
        RuntimeError {
            kind: error.kind.as_str().to_string(),
            reason: error.reason,
        }
    }
}

impl From<MinSqliteWrapperError<'_>> for PackageError {
    #[inline(always)]
    fn from(error: MinSqliteWrapperError) -> Self {
        PackageErrorKind::InstallationFailed(Some(error.reason)).throw()
    }
}
