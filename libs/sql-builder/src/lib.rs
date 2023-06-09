use std::fmt::Display;

pub struct Column(pub(crate) String, pub(crate) usize);

impl Column {
    #[inline(always)]
    pub fn new(name: String, prepared_id: usize) -> Self {
        Self(name, prepared_id)
    }
}

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
    Insert(String, Option<Vec<Column>>),
    /// 1st arg: "INTO"
    /// 2nd arg: "SELECT"
    InsertFromSelect(String, select::Select),
    /// 1st arg: Table name to be updated.
    /// 2nd arg: Column names and prepared statement ids.
    Update(String, Vec<Column>),
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
                    writeln!(
                        f,
                        "At least one column must be defined for DISTINCT queries."
                    )?;
                    return Err(std::fmt::Error);
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
            Operation::Update(table, columns) => {
                let set_fields: Vec<String> = columns
                    .iter()
                    .map(|column| format!("{} = ?{}", column.0, column.1))
                    .collect();
                let set_fields = set_fields.join(", ");

                write!(f, "UPDATE {} SET {}", table, set_fields)
            }
        }
    }
}

pub enum Where {
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    Equal(usize, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    NotEqual(usize, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    LessThan(usize, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    LessThanOrEqual(usize, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    GreaterThan(usize, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    GreaterThanOrEqual(usize, String),
    /// 1st and 2nd args: Prepared statement id for later value binding
    /// 3rd arg: Column name
    Between(usize, usize, String),
    /// 1st and 2nd args: Prepared statement id for later value binding
    /// 3rd arg: Column name
    NotBetween(usize, usize, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    In(Vec<usize>, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    NotIn(Vec<usize>, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    Like(usize, String),
    /// 1st arg: Prepared statement id for later value binding
    /// 2nd arg: Column name
    NotLike(usize, String),
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

            Where::In(pre_ids, name) => {
                let pre_ids: Vec<String> = pre_ids.iter().map(|id| format!("?{}", id)).collect();
                let pre_ids = pre_ids.join(", ");

                write!(f, "{} IN ({})", name, pre_ids)
            }

            Where::NotIn(pre_ids, name) => {
                let pre_ids: Vec<String> = pre_ids.iter().map(|id| format!("?{}", id)).collect();
                let pre_ids = pre_ids.join(", ");

                write!(f, "{} NOT IN ({})", name, pre_ids)
            }

            Where::Like(index, name) => write!(f, "{} LIKE ?{}", name, index),

            Where::NotLike(index, name) => write!(f, "{} NOT LIKE ?{}", name, index),
        }
    }
}

pub mod delete;
pub mod insert;
pub mod select;
pub mod update;
