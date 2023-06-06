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
    Ok = 0,

    // 100-199 Package related errors
    PackageError_InvalidPackageFiles = 100,
    PackageError_UnsupportedPackageArchitecture = 101,
    PackageError_UnsupportedChecksumAlgorithm = 102,
    PackageError_InstallationFailed = 103,
    PackageError_UnsupportedStandard = 104,
    PackageError_DeletionFailed = 105,
    PackageError_AlreadyInstalled = 106,
    PackageError_DoesNotExists = 107,
    PackageError_UnrecognizedRepository = 108,
    PackageError_DbOperationFailed = 109,
    PackageError_FailedExecutingStage1Script = 110,

    // 200-299 Module related errors
    ModuleError_DynamicLibraryNotFound = 200,
    ModuleError_EntrypointFunctionNotFound = 201,
    ModuleError_Internal = 202,
    ModuleError_ModuleNotFound = 203,
    ModuleError_ModuleAlreadyExists = 204,

    // 300-399 IO related errors
    IoError = 300,
    IoError_NotFound = 301,
    IoError_PermissionDenied = 303,
    IoError_ConnectionRefused = 304,
    IoError_ConnectionReset = 305,
    IoError_ConnectionAborted = 306,
    IoError_NotConnected = 307,
    IoError_AddrInUse = 308,
    IoError_AddrNotAvailable = 309,
    IoError_BrokenPipe = 310,
    IoError_AlreadyExists = 311,
    IoError_WouldBlock = 312,
    IoError_InvalidInput = 313,
    IoError_InvalidData = 314,
    IoError_TimedOut = 315,
    IoError_WriteZero = 316,
    IoError_Interrupted = 317,
    IoError_Unsupported = 318,
    IoError_UnexpectedEof = 319,
    IoError_OutOfMemory = 320,

    // 400-499 Database related errors
    SqlError_FailedExecuting = 400,
    SqlError_FailedPreparedExecuting = 401,
    SqlError_FailedParameterBinding = 402,
    SqlError_WrapperLibError = 403,
    SqlError_MigrationError = 404,
    MinSqliteWrapperError = 405,

    // 500-599 Repository related errors
    RepositoryError_RepositoryNotFound = 500,
    RepositoryError_RepositoryAlreadyExists = 501,
    RepositoryError_Internal = 502,
    RepositoryError_PackageNotFound = 503,

    // 900-999 ABI related errors
    Str_Utf8Error = 900,
    CStr_NulError = 901,
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
            "PackageError_FailedExecutingStage1Script" => {
                Self::PackageError_FailedExecutingStage1Script
            }

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

            "Str_Utf8Error" => Self::Str_Utf8Error,

            "CStr_NulError" => Self::CStr_NulError,

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
pub mod repository;
