use crate::{
    lpm::LpmError,
    pkg::{PackageError, PackageErrorKind},
    ErrorCommons, MainError,
};

#[cfg(feature = "sdk")]
use crate::ResultCode;

use min_sqlite3_sys::prelude::{MinSqliteWrapperError, SqlitePrimaryResult};

#[macro_export]
macro_rules! try_bind_val {
    ($sql: expr, $c_index: expr, $val: expr) => {
        let status = $sql.bind_val($c_index, $val);
        if status != min_sqlite3_sys::prelude::SqlitePrimaryResult::Ok {
            $sql.kill();

            return Err(ehandle::db::SqlErrorKind::FailedParameterBinding(
                $c_index,
                format!("{:?}", $val),
                status,
            )
            .to_lpm_err()
            .into());
        }
    };
}

#[macro_export]
macro_rules! try_execute_prepared {
    ($sql: expr, $err: expr) => {
        match $sql.execute_prepared() {
            min_sqlite3_sys::prelude::PreparedStatementStatus::FoundRow => {
                min_sqlite3_sys::prelude::PreparedStatementStatus::FoundRow
            }
            min_sqlite3_sys::prelude::PreparedStatementStatus::Done => {
                min_sqlite3_sys::prelude::PreparedStatementStatus::Done
            }
            _ => {
                $sql.kill();
                return Err(ehandle::db::SqlErrorKind::FailedPreparedExecuting($err)
                    .to_lpm_err()
                    .into());
            }
        }
    };
}

#[macro_export]
macro_rules! try_execute {
    ($db: expr, $statement: expr) => {
        match $db.execute($statement.clone(), super::SQL_NO_CALLBACK_FN)? {
            min_sqlite3_sys::prelude::SqlitePrimaryResult::Ok => SqlitePrimaryResult::Ok,
            e => {
                return Err(ehandle::db::SqlErrorKind::FailedExecuting($statement, e)
                    .to_lpm_err()
                    .into());
            }
        }
    };
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum MigrationErrorKind {
    VersionCouldNotSet,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum SqlErrorKind {
    FailedExecuting(String, SqlitePrimaryResult),
    FailedPreparedExecuting(String),
    FailedParameterBinding(usize, String, SqlitePrimaryResult),
    WrapperLibError(String, String),
    MigrationError(MigrationErrorKind),
}

#[derive(Debug)]
pub struct SqlError {
    pub(crate) kind: String,
    pub(crate) reason: String,
}

impl ErrorCommons for SqlErrorKind {
    type Error = SqlError;

    fn as_str(&self) -> &str {
        match self {
            Self::FailedExecuting(..) => "FailedExecuting",
            Self::FailedPreparedExecuting(_) => "FailedPreparedExecuting",
            Self::FailedParameterBinding(..) => "FailedParameterBinding",
            SqlErrorKind::WrapperLibError(..) => "WrapperLibError",
            SqlErrorKind::MigrationError(_) => "MigrationError",
        }
    }

    fn to_err(&self) -> Self::Error {
        match self {
            Self::FailedExecuting(ref statement, ref status) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!(
                    "Failed executing '{}' statement. Error status: {:?}.",
                    statement, status
                ),
            },
            Self::FailedPreparedExecuting(ref error) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: error.clone(),
            },
            Self::FailedParameterBinding(ref param_index, ref param_value, ref result) => {
                Self::Error {
                    kind: self.as_str().to_owned(),
                    reason: format!(
                        "Failed binding '{}' value on {} index. Error: {:?}",
                        param_value, param_index, result
                    ),
                }
            }
            SqlErrorKind::WrapperLibError(ref kind, ref reason) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!(
                    "'{}' occurred from the wrapper library. Reason: '{}'.",
                    kind, reason
                ),
            },
            SqlErrorKind::MigrationError(ref error) => Self::Error {
                kind: self.as_str().to_owned(),
                reason: format!(
                    "Migration process could not be completed. Error: '{:?}'",
                    error
                ),
            },
        }
    }

    #[inline]
    #[cfg(feature = "sdk")]
    fn to_lpm_err(&self) -> LpmError<Self::Error> {
        LpmError::new(self.to_err(), self.to_result_code())
    }

    #[inline]
    #[cfg(not(feature = "sdk"))]
    fn to_lpm_err(&self) -> LpmError<Self::Error> {
        LpmError::new(self.to_err())
    }

    #[cfg(feature = "sdk")]
    fn to_result_code(&self) -> ResultCode {
        match self {
            SqlErrorKind::FailedExecuting(_, _) => ResultCode::SqlError_FailedExecuting,
            SqlErrorKind::FailedPreparedExecuting(_) => {
                ResultCode::SqlError_FailedPreparedExecuting
            }
            SqlErrorKind::FailedParameterBinding(_, _, _) => {
                ResultCode::SqlError_FailedParameterBinding
            }
            SqlErrorKind::WrapperLibError(_, _) => ResultCode::SqlError_WrapperLibError,
            SqlErrorKind::MigrationError(_) => ResultCode::SqlError_MigrationError,
        }
    }
}

impl From<MinSqliteWrapperError<'_>> for LpmError<MainError> {
    #[track_caller]
    fn from(error: MinSqliteWrapperError) -> Self {
        LpmError::new(
            MainError {
                kind: error.kind.to_owned(),
                reason: error.reason,
            },
            #[cfg(feature = "sdk")]
            ResultCode::MinSqliteWrapperError,
        )
    }
}

impl From<LpmError<SqlError>> for LpmError<MainError> {
    #[track_caller]
    #[cfg(feature = "sdk")]
    fn from(error: LpmError<SqlError>) -> Self {
        let e = MainError {
            kind: error.error_type.kind,
            reason: error.error_type.reason,
        };

        let result_tag = "SqlError";
        let result_code = ResultCode::from_str(&format!("{}_{}", result_tag, &e.kind));
        LpmError::new_with_traces(e, result_code, error.chain)
    }

    #[track_caller]
    #[cfg(not(feature = "sdk"))]
    fn from(error: LpmError<SqlError>) -> Self {
        let e = MainError {
            kind: error.error_type.kind,
            reason: error.error_type.reason,
        };

        LpmError::new_with_traces(e, error.chain)
    }
}

impl From<LpmError<SqlError>> for LpmError<PackageError> {
    #[track_caller]
    #[cfg(feature = "sdk")]
    fn from(error: LpmError<SqlError>) -> Self {
        let e = PackageErrorKind::DbOperationFailed(error.error_type.reason).to_err();
        LpmError::new_with_traces(e, ResultCode::from_str(&error.error_type.kind), error.chain)
    }

    #[track_caller]
    #[cfg(not(feature = "sdk"))]
    fn from(error: LpmError<SqlError>) -> Self {
        let e = PackageErrorKind::DbOperationFailed(error.error_type.reason).to_err();
        LpmError::new_with_traces(e, error.chain)
    }
}

impl From<MinSqliteWrapperError<'_>> for LpmError<SqlError> {
    #[track_caller]
    fn from(error: MinSqliteWrapperError) -> Self {
        SqlErrorKind::WrapperLibError(error.kind.to_string(), error.reason).to_lpm_err()
    }
}
