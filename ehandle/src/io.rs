use crate::RuntimeError;
use std::io::{self, ErrorKind};

impl From<io::Error> for RuntimeError {
    #[inline(always)]
    fn from(error: io::Error) -> Self {
        RuntimeError {
            kind: parse_io_error_kind(error.kind()).to_string(),
            reason: error.to_string(),
        }
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
