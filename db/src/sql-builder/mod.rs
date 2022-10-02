use std::fmt::Display;

pub trait CommonInstructions {
    /// Returns constructed SQL statement in String form
    fn to_string(&self) -> String;
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

pub(crate) enum Operation {
    /// 1st arg: Vector of column names. None and empty vector means "*".
    /// 2nd arg: "FROM"
    Select(Option<Vec<String>>, String),
    /// 1st arg: Vector of column names. None means "*".
    /// 2nd arg: "FROM"
    SelectDistinct(Vec<String>, String),
    /// 1st arg: "FROM"
    Delete(String),
    /// 1st arg: "INTO"
    /// 2nd arg: Column names and prepared statement ids.
    Insert(String, Option<Vec<insert::Column>>),
    /// 1st arg: "INTO"
    /// 2nd arg: "SELECT"
    InsertFromSelect(String, select::Select),
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Select(columns, table) => {
                let columns = match columns {
                    Some(columns) if !columns.is_empty() => columns.join(", "),
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
            Operation::Delete(table) => {
                write!(f, "DELETE FROM {}", table)
            }
            Operation::Insert(table, columns) => match columns {
                Some(columns) if !columns.is_empty() => {
                    let prepared_values: Vec<String> = columns
                        .iter()
                        .map(|column| format!("?{}", column.1))
                        .collect();
                    let prepared_values = prepared_values.join(", ");

                    let columns: Vec<&str> =
                        columns.iter().map(|column| column.0.as_str()).collect();
                    let columns = columns.join(", ");

                    write!(
                        f,
                        "INSERT INTO {} ({}) VALUES({})",
                        table, columns, prepared_values
                    )
                }
                _ => {
                    write!(f, "INSERT INTO {} DEFAULT VALUES", table)
                }
            },
            Operation::InsertFromSelect(table, select) => {
                write!(f, "INSERT INTO {} {}", table, select.0)
            }
        }
    }
}

pub enum Where {
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    Equal(u8, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    NotEqual(u8, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    LessThan(u8, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    LessThanOrEqual(u8, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    GreaterThan(u8, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    GreaterThanOrEqual(u8, String),
    /// 1st and 2nd args: Prepared statement id for later value binding
    /// 3rd arg: Column name
    Between(u8, u8, String),
    /// 1st and 2nd args: Prepared statement id for later value binding
    /// 3rd arg: Column name
    NotBetween(u8, u8, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    In(u8, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    NotIn(u8, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    Like(u8, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    NotLike(u8, String),
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

pub mod delete;
pub mod insert;
pub mod select;
