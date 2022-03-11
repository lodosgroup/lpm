use crate::RuntimeError;

#[non_exhaustive]
#[derive(Debug)]
pub enum PackageErrorKind {
    InvalidPackageFiles,
    UnsupportedChecksumAlgorithm,
}

impl PackageErrorKind {
    pub fn as_str(&self) -> &str {
        match self {
            PackageErrorKind::InvalidPackageFiles => "InvalidPackageFiles",
            PackageErrorKind::UnsupportedChecksumAlgorithm => "UnsupportedChecksumAlgorithm",
        }
    }
}

#[derive(Debug)]
pub struct PackageError {
    pub kind: PackageErrorKind,
    pub reason: String,
}

impl PackageError {
    pub fn new(kind: PackageErrorKind) -> Self {
        match kind {
            PackageErrorKind::InvalidPackageFiles => PackageError {
                kind,
                reason: "According to the checksum file, the package files are not valid."
                    .to_string(),
            },
            PackageErrorKind::UnsupportedChecksumAlgorithm => PackageError {
                kind,
                reason: "The checksum algorithm of the package is not supported.".to_string(),
            },
        }
    }
}

impl From<PackageError> for RuntimeError {
    fn from(error: PackageError) -> Self {
        RuntimeError {
            kind: error.kind.as_str().to_string(),
            reason: error.reason,
        }
    }
}
