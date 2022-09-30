use std::fmt::Display;

pub trait CommonInstructions {
    /// Returns constructed SQL statement in String form
    fn to_string(&self) -> String;
}

pub enum Operation {
    // 1st arg: Vector of column names. None means "*".
    // 2nd arg: Arg for "FROM".
    Select(Option<Vec<String>>, String),
    SelectDistinct(Vec<String>, String),
    Create(String, Option<CreateOperationArg>),
    Update(String),
    Delete(String),
    Replace(String),
    Insert(String, Vec<String>),
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Select(columns, table) => {
                let columns = match columns {
                    Some(t) if !t.is_empty() => t.join(", "),
                    _ => String::from("*"),
                };

                write!(f, "SELECT {} FROM {}", columns, table)
            }
            Operation::SelectDistinct(columns, table) => {
                if columns.is_empty() {
                    common::log_and_panic!(
                        "At least one column must be defined for DISTINCT queries."
                    );
                }

                let columns = columns.join(", ");

                write!(f, "SELECT DISTINCT {} FROM {}", columns, table)
            }
            Operation::Create(table, arg) => {
                if let Some(arg) = arg {
                    write!(f, "CREATE TABLE {} {}", arg, table)
                } else {
                    write!(f, "CREATE TABLE {}", table)
                }
            }
            Operation::Update(table) => {
                write!(f, "UPDATE {}", table)
            }
            Operation::Delete(table) => {
                write!(f, "DELETE FROM {}", table)
            }
            Operation::Replace(_) => todo!(),
            Operation::Insert(table, columns) => {
                if columns.is_empty() {
                    common::log_and_panic!(
                        "No columns were detected for inserting to table {}",
                        table
                    );
                }

                let prepared_values: Vec<&str> = columns.iter().map(|_| "?").collect();
                let prepared_values = prepared_values.join(", ");
                let columns = columns.join(",");

                write!(
                    f,
                    "INSERT INTO {} ({}) VALUES({})",
                    table, columns, prepared_values
                )
            }
        }
    }
}

pub enum CreateOperationArg {
    IfNotExists,
}

impl Display for CreateOperationArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateOperationArg::IfNotExists => write!(f, "IF NOT EXISTS"),
        }
    }
}

/// Column's index to bind value following with it's name
pub enum Where {
    Equal(u8, String),
    NotEqual(u8, String),
    LessThan(u8, String),
    LessThanOrEqual(u8, String),
    GreaterThan(u8, String),
    GreaterThanOrEqual(u8, String),
    Between(u8, u8, String),
    NotBetween(u8, u8, String),
    In(u8, String),
    NotIn(u8, String),
    Like(u8, String),
    NotLike(u8, String),
}

pub trait WhereInstructions {
    /// Adds '('
    fn open_parentheses(&self) -> Self;

    /// Adds ')'
    fn close_parentheses(&self) -> Self;

    /// Only adds 'AND' keyword
    fn and_keyword(&self) -> Self;

    /// Only adds 'OR' keyword
    fn or_keyword(&self) -> Self;

    /// Adds contiditon
    fn where_condition(&self, w: Where) -> Self;

    /// Adds contiditon as 'AND'
    fn and_where(&self, w: Where) -> Self;

    /// Adds contiditon as 'OR'
    fn or_where(&self, w: Where) -> Self;
}

impl Display for Where {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Where::Equal(index, name) => write!(f, "{} = ?{}", name, index),

            Where::NotEqual(index, name) => write!(f, "{} != ?{}", name, index),

            Where::LessThan(index, name) => write!(f, "{} < ?{}", name, index),

            Where::LessThanOrEqual(index, name) => write!(f, "{} <= ?{}", name, index),

            Where::GreaterThan(index, name) => write!(f, "{} > ?{}", name, index),

            Where::GreaterThanOrEqual(index, name) => write!(f, "{} >= ?{}", name, index),

            Where::Between(index1, index2, name) => {
                write!(f, "{} BETWEEN ?{} AND ?{}", name, index1, index2)
            }

            Where::NotBetween(index1, index2, name) => {
                write!(f, "{} NOT BETWEEN ?{} AND ?{}", name, index1, index2)
            }

            Where::In(index, name) => write!(f, "{} IN ?{}", name, index),

            Where::NotIn(index, name) => write!(f, "{} NOT IN ?{}", name, index),

            Where::Like(index, name) => write!(f, "{} LIKE ?{}", name, index),

            Where::NotLike(index, name) => write!(f, "{} NOT LIKE ?{}", name, index),
        }
    }
}

mod select;
