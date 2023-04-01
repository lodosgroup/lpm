#[cfg(feature = "sdk")]
use crate::ResultCode;
use crate::{lpm::LpmError, MainError};

use std::io;

#[cfg(feature = "sdk")]
impl From<io::ErrorKind> for ResultCode {
    fn from(kind: io::ErrorKind) -> Self {
        match kind {
            io::ErrorKind::NotFound => ResultCode::IoError_NotFound,
            io::ErrorKind::PermissionDenied => ResultCode::IoError_PermissionDenied,
            io::ErrorKind::ConnectionRefused => ResultCode::IoError_ConnectionRefused,
            io::ErrorKind::ConnectionReset => ResultCode::IoError_ConnectionReset,
            // io::ErrorKind::HostUnreachable => todo!(),
            // io::ErrorKind::NetworkUnreachable => todo!(),
            io::ErrorKind::ConnectionAborted => ResultCode::IoError_ConnectionAborted,
            io::ErrorKind::NotConnected => ResultCode::IoError_NotConnected,
            io::ErrorKind::AddrInUse => ResultCode::IoError_AddrInUse,
            io::ErrorKind::AddrNotAvailable => ResultCode::IoError_AddrNotAvailable,
            // io::ErrorKind::NetworkDown => todo!(),
            io::ErrorKind::BrokenPipe => ResultCode::IoError_BrokenPipe,
            io::ErrorKind::AlreadyExists => ResultCode::IoError_AlreadyExists,
            io::ErrorKind::WouldBlock => ResultCode::IoError_WouldBlock,
            // io::ErrorKind::NotADirectory => todo!(),
            // io::ErrorKind::IsADirectory => todo!(),
            // io::ErrorKind::DirectoryNotEmpty => todo!(),
            // io::ErrorKind::ReadOnlyFilesystem => todo!(),
            // io::ErrorKind::FilesystemLoop => todo!(),
            // io::ErrorKind::StaleNetworkFileHandle => todo!(),
            io::ErrorKind::InvalidInput => ResultCode::IoError_InvalidInput,
            io::ErrorKind::InvalidData => ResultCode::IoError_InvalidData,
            io::ErrorKind::TimedOut => ResultCode::IoError_TimedOut,
            io::ErrorKind::WriteZero => ResultCode::IoError_WriteZero,
            // io::ErrorKind::StorageFull => todo!(),
            // io::ErrorKind::NotSeekable => todo!(),
            // io::ErrorKind::FilesystemQuotaExceeded => todo!(),
            // io::ErrorKind::FileTooLarge => todo!(),
            // io::ErrorKind::ResourceBusy => todo!(),
            // io::ErrorKind::ExecutableFileBusy => todo!(),
            // io::ErrorKind::Deadlock => todo!(),
            // io::ErrorKind::CrossesDevices => todo!(),
            // io::ErrorKind::TooManyLinks => todo!(),
            // io::ErrorKind::InvalidFilename => todo!(),
            // io::ErrorKind::ArgumentListTooLong => todo!(),
            io::ErrorKind::Interrupted => ResultCode::IoError_Interrupted,
            io::ErrorKind::Unsupported => ResultCode::IoError_Unsupported,
            io::ErrorKind::UnexpectedEof => ResultCode::IoError_UnexpectedEof,
            io::ErrorKind::OutOfMemory => ResultCode::IoError_OutOfMemory,
            _other => ResultCode::IoError,
        }
    }
}

impl From<io::Error> for LpmError<io::Error> {
    #[track_caller]
    #[cfg(feature = "sdk")]
    fn from(error: io::Error) -> Self {
        let kind = error.kind();
        LpmError::new(error, kind.into())
    }

    #[track_caller]
    #[cfg(not(feature = "sdk"))]
    fn from(error: io::Error) -> Self {
        LpmError::new(error)
    }
}

impl From<io::Error> for LpmError<MainError> {
    #[track_caller]
    #[cfg(feature = "sdk")]
    fn from(error: io::Error) -> Self {
        LpmError::new(
            MainError {
                kind: error.kind().to_string(),
                reason: error.to_string(),
            },
            error.kind().into(),
        )
    }

    #[track_caller]
    #[cfg(not(feature = "sdk"))]
    fn from(error: io::Error) -> Self {
        LpmError::new(MainError {
            kind: error.kind().to_string(),
            reason: error.to_string(),
        })
    }
}

impl From<LpmError<io::Error>> for LpmError<MainError> {
    #[track_caller]
    #[cfg(feature = "sdk")]
    fn from(error: LpmError<io::Error>) -> Self {
        let e = MainError {
            kind: error.error_type.kind().to_string(),
            reason: error.error_type.to_string(),
        };

        LpmError::new_with_traces(e, error.error_type.kind().into(), error.chain)
    }

    #[track_caller]
    #[cfg(not(feature = "sdk"))]
    fn from(error: LpmError<io::Error>) -> Self {
        let e = MainError {
            kind: error.error_type.kind().to_string(),
            reason: error.error_type.to_string(),
        };

        LpmError::new_with_traces(e, error.chain)
    }
}
