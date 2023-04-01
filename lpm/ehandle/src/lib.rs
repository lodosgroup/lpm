use lpm::LpmError;

#[macro_export]
macro_rules! simple_e_fmt {
    ($format: expr, $($args: tt)+) => { format!($format, $($args)+) };
    ($format: expr) => { format!($format) }
}

#[repr(u16)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[cfg(feature = "sdk")]
pub enum ResultCode {
    Ok,

    SqlError_FailedExecuting,
    SqlError_FailedPreparedExecuting,
    SqlError_FailedParameterBinding,
    SqlError_WrapperLibError,
    SqlError_MigrationError,

    ModuleError_DynamicLibraryNotFound,
    ModuleError_EntrypointFunctionNotFound,
    ModuleError_Internal,

    PackageError_InvalidPackageFiles,
    PackageError_UnsupportedPackageArchitecture,
    PackageError_UnsupportedChecksumAlgorithm,
    PackageError_InstallationFailed,
    PackageError_UnsupportedStandard,
    PackageError_DeletionFailed,
    PackageError_AlreadyInstalled,
    PackageError_DoesNotExists,
    PackageError_UnrecognizedRepository,
    PackageError_DbOperationFailed,
    PackageError_PackageKindNotFound,

    MinSqliteWrapperError,

    IoError,
    IoError_NotFound,
    IoError_PermissionDenied,
    IoError_ConnectionRefused,
    IoError_ConnectionReset,
    IoError_ConnectionAborted,
    IoError_NotConnected,
    IoError_AddrInUse,
    IoError_AddrNotAvailable,
    IoError_BrokenPipe,
    IoError_AlreadyExists,
    IoError_WouldBlock,
    IoError_InvalidInput,
    IoError_InvalidData,
    IoError_TimedOut,
    IoError_WriteZero,
    IoError_Interrupted,
    IoError_Unsupported,
    IoError_UnexpectedEof,
    IoError_OutOfMemory,
}

#[cfg(feature = "sdk")]
impl ResultCode {
    fn from_str(kind: &str) -> Self {
        match kind {
            "Ok" => Self::Ok,

            "SqlError_FailedExecuting" => Self::SqlError_FailedExecuting,
            "SqlError_FailedPreparedExecuting" => Self::SqlError_FailedPreparedExecuting,
            "SqlError_FailedParameterBinding" => Self::SqlError_FailedParameterBinding,
            "SqlError_WrapperLibError" => Self::SqlError_WrapperLibError,
            "SqlError_MigrationError" => Self::SqlError_MigrationError,

            "ModuleError_Internal" => Self::ModuleError_Internal,
            "ModuleError_EntrypointFunctionNotFound" => {
                Self::ModuleError_EntrypointFunctionNotFound
            }
            "PackageError_UnsupportedPackageArchitecture" => {
                Self::PackageError_UnsupportedPackageArchitecture
            }
            "PackageError_UnsupportedChecksumAlgorithm" => {
                Self::PackageError_UnsupportedChecksumAlgorithm
            }

            "PackageError_InvalidPackageFiles" => Self::PackageError_InvalidPackageFiles,
            "ModuleError_DynamicLibraryNotFound" => Self::ModuleError_DynamicLibraryNotFound,
            "PackageError_InstallationFailed" => Self::PackageError_InstallationFailed,
            "PackageError_UnsupportedStandard" => Self::PackageError_UnsupportedStandard,
            "PackageError_DeletionFailed" => Self::PackageError_DeletionFailed,
            "PackageError_AlreadyInstalled" => Self::PackageError_AlreadyInstalled,
            "PackageError_DoesNotExists" => Self::PackageError_DoesNotExists,
            "PackageError_UnrecognizedRepository" => Self::PackageError_UnrecognizedRepository,
            "PackageError_DbOperationFailed" => Self::PackageError_DbOperationFailed,
            "PackageError_PackageKindNotFound" => Self::PackageError_PackageKindNotFound,

            "MinSqliteWrapperError" => Self::MinSqliteWrapperError,

            "IoError" => Self::IoError,
            "IoError_NotFound" => Self::IoError_NotFound,
            "IoError_PermissionDenied" => Self::IoError_PermissionDenied,
            "IoError_ConnectionRefused" => Self::IoError_ConnectionRefused,
            "IoError_ConnectionReset" => Self::IoError_ConnectionReset,
            "IoError_ConnectionAborted" => Self::IoError_ConnectionAborted,
            "IoError_NotConnected" => Self::IoError_NotConnected,
            "IoError_AddrInUse" => Self::IoError_AddrInUse,
            "IoError_AddrNotAvailable" => Self::IoError_AddrNotAvailable,
            "IoError_BrokenPipe" => Self::IoError_BrokenPipe,
            "IoError_AlreadyExists" => Self::IoError_AlreadyExists,
            "IoError_WouldBlock" => Self::IoError_WouldBlock,
            "IoError_InvalidInput" => Self::IoError_InvalidInput,
            "IoError_InvalidData" => Self::IoError_InvalidData,
            "IoError_TimedOut" => Self::IoError_TimedOut,
            "IoError_WriteZero" => Self::IoError_WriteZero,
            "IoError_Interrupted" => Self::IoError_Interrupted,
            "IoError_Unsupported" => Self::IoError_Unsupported,
            "IoError_UnexpectedEof" => Self::IoError_UnexpectedEof,
            "IoError_OutOfMemory" => Self::IoError_OutOfMemory,

            other => {
                panic!("Invalid result type '{}'.", other);
            }
        }
    }
}

pub trait ErrorCommons {
    type Error;

    fn as_str(&self) -> &str;
    fn to_err(&self) -> Self::Error;
    #[track_caller]
    fn to_lpm_err(&self) -> LpmError<Self::Error>;
    #[cfg(feature = "sdk")]
    fn to_result_code(&self) -> ResultCode;
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct MainError {
    kind: String,
    reason: String,
}

pub mod db;
mod io;
pub mod lpm;
pub mod module;
pub mod pkg;
