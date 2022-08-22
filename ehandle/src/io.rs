use crate::{lpm::LpmError, RuntimeError};
use std::io::{self, ErrorKind};

impl From<io::Error> for LpmError<io::Error> {
    #[track_caller]
    fn from(error: io::Error) -> Self {
        LpmError::new(error)
    }
}

impl From<io::Error> for LpmError<RuntimeError> {
    #[track_caller]
    fn from(error: io::Error) -> Self {
        LpmError::new(RuntimeError {
            kind: parse_io_error_kind(error.kind()).to_string(),
            reason: error.to_string(),
        })
    }
}

impl From<LpmError<io::Error>> for LpmError<RuntimeError> {
    #[track_caller]
    fn from(error: LpmError<io::Error>) -> Self {
        let e = RuntimeError {
            kind: parse_io_error_kind(error.error_type.kind()).to_string(),
            reason: error.error_type.to_string(),
        };
        LpmError::new_with_traces(e, error.error_stack)
    }
}

#[inline(always)]
fn parse_io_error_kind(kind: ErrorKind) -> &'static str {
    match &kind {
        ErrorKind::NotFound => "NotFound",
        ErrorKind::PermissionDenied => "PermissionDenied",
        ErrorKind::ConnectionRefused => "ConnectionRefused",
        ErrorKind::ConnectionReset => "ConnectionReset",
        ErrorKind::HostUnreachable => "HostUnreachable",
        ErrorKind::NetworkUnreachable => "NetworkUnreachable",
        ErrorKind::ConnectionAborted => "ConnectionAborted",
        ErrorKind::NotConnected => "NotConnected",
        ErrorKind::AddrInUse => "AddInUse",
        ErrorKind::AddrNotAvailable => "AddrNotAvailable",
        ErrorKind::NetworkDown => "NetworkDown",
        ErrorKind::BrokenPipe => "BrokenPipe",
        ErrorKind::AlreadyExists => "AlreadyExists",
        ErrorKind::WouldBlock => "WouldBlock",
        ErrorKind::NotADirectory => "NotADirectory",
        ErrorKind::IsADirectory => "IsADirectory",
        ErrorKind::DirectoryNotEmpty => "DirectoryNotEmpty",
        ErrorKind::ReadOnlyFilesystem => "ReadOnlyFileSystem",
        ErrorKind::FilesystemLoop => "FilesystemLoop",
        ErrorKind::StaleNetworkFileHandle => "StaleNetworkFileHandle",
        ErrorKind::InvalidInput => "InvalidInput",
        ErrorKind::InvalidData => "InvaludData",
        ErrorKind::TimedOut => "TimedOut",
        ErrorKind::WriteZero => "WriteZero",
        ErrorKind::StorageFull => "StorageFull",
        ErrorKind::NotSeekable => "NotSeekable",
        ErrorKind::FilesystemQuotaExceeded => "FilesystemQuotaExceeded",
        ErrorKind::FileTooLarge => "FileTooLarge",
        ErrorKind::ResourceBusy => "ResourceBusy",
        ErrorKind::ExecutableFileBusy => "ExecutableFileBusy",
        ErrorKind::Deadlock => "Deadlock",
        ErrorKind::CrossesDevices => "CrossDevices",
        ErrorKind::TooManyLinks => "TooManyLinks",
        ErrorKind::ArgumentListTooLong => "ArgumentListTooLong",
        ErrorKind::Interrupted => "Interrupted",
        ErrorKind::Unsupported => "Unsopperted",
        ErrorKind::UnexpectedEof => "UnexpectedEof",
        ErrorKind::OutOfMemory => "OutOfMemory",
        ErrorKind::Uncategorized => "Uncategorized",
        ErrorKind::InvalidFilename => "InvalidFilename",
        ErrorKind::Other => "Other",
        _ => "UnrecognizedErrorKind",
    }
}
