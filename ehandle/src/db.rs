use crate::{
    lpm::LpmError,
    pkg::{PackageError, PackageErrorKind},
    ErrorCommons, MainError,
};
use min_sqlite3_sys::prelude::{MinSqliteWrapperError, SqlitePrimaryResult};

#[macro_export]
macro_rules! try_bind_val {
    ($sql: expr, $c_index: expr, $val: expr) => {
        let status = $sql.bind_val($c_index, $val);
        if status != min_sqlite3_sys::prelude::SqlitePrimaryResult::Ok {
            $sql.kill();

            return Err(ehandle::lpm::LpmError::new(
                ehandle::db::SqlErrorKind::FailedParameterBinding(
                    $c_index,
                    format!("{:?}", $val),
                    status,
                )
                .throw(),
            )
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
                return Err(ehandle::lpm::LpmError::new(
                    ehandle::db::SqlErrorKind::FailedPreparedExecuting($err).throw(),
                )
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
                return Err(ehandle::lpm::LpmError::new(
                    ehandle::db::SqlErrorKind::FailedExecuting($statement, e).throw(),
                )
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
    kind: SqlErrorKind,
    reason: String,
}

impl ErrorCommons<SqlError> for SqlErrorKind {
    fn as_str(&self) -> &str {
        match self {
            Self::FailedExecuting(..) => "FailedExecuting",
            Self::FailedPreparedExecuting(_) => "FailedPreparedExecuting",
            Self::FailedParameterBinding(..) => "FailedParameterBinding",
            SqlErrorKind::WrapperLibError(..) => "WrapperLibError",
            SqlErrorKind::MigrationError(_) => "MigrationError",
        }
    }

    fn throw(&self) -> SqlError {
        match self {
            Self::FailedExecuting(ref statement, ref status) => SqlError {
                kind: self.clone(),
                reason: format!(
                    "Failed executing '{}' statement. Error status: {:?}.",
                    statement, status
                ),
            },
            Self::FailedPreparedExecuting(ref error) => SqlError {
                kind: self.clone(),
                reason: error.clone(),
            },
            Self::FailedParameterBinding(ref param_index, ref param_value, ref result) => {
                SqlError {
                    kind: self.clone(),
                    reason: format!(
                        "Failed binding '{}' value on {} index. Error: {:?}",
                        param_value, param_index, result
                    ),
                }
            }
            SqlErrorKind::WrapperLibError(ref kind, ref reason) => SqlError {
                kind: self.clone(),
                reason: format!(
                    "'{}' occurred from the wrapper library. Reason: '{}'.",
                    kind, reason
                ),
            },
            SqlErrorKind::MigrationError(ref error) => SqlError {
                kind: self.clone(),
                reason: format!(
                    "Migration process could not be completed. Error: '{:?}'",
                    error
                ),
            },
        }
    }
}

impl From<MinSqliteWrapperError<'_>> for LpmError<MainError> {
    #[track_caller]
    fn from(error: MinSqliteWrapperError) -> Self {
        LpmError::new(MainError {
            kind: error.kind.to_string(),
            reason: error.reason,
        })
    }
}

impl From<LpmError<SqlError>> for LpmError<MainError> {
    #[track_caller]
    fn from(error: LpmError<SqlError>) -> Self {
        let e = MainError {
            kind: error.error_type.kind.as_str().to_string(),
            reason: error.error_type.reason,
        };

        LpmError::new_with_traces(e, error.chain)
    }
}

impl From<LpmError<SqlError>> for LpmError<PackageError> {
    #[track_caller]
    fn from(error: LpmError<SqlError>) -> Self {
        let e = PackageErrorKind::InstallationFailed(Some(error.error_type.reason)).throw();
        LpmError::new_with_traces(e, error.chain)
    }
}

impl From<MinSqliteWrapperError<'_>> for LpmError<SqlError> {
    #[track_caller]
    fn from(error: MinSqliteWrapperError) -> Self {
        LpmError::new(SqlErrorKind::WrapperLibError(error.kind.to_string(), error.reason).throw())
    }
}
