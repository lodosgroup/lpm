use std::fmt::Display;

pub struct SqlBuilder {}

pub enum Operation {
    // 1st arg: Vector of column names. None means "*".
    // 2nd arg: Arg for "FROM".
    Select(Option<Vec<String>>, String),
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
                    Some(t) if !t.is_empty() => t.join(","),
                    _ => String::from("*"),
                };

                write!(f, "SELECT {} FROM {}", columns, table)
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

                let columns = columns.join(",");

                // TODO
                // handle column values
                write!(f, "INSERT INTO {} ({}) VALUES", table, columns)
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

pub enum ColVal {
    Null,
    SignedNumber(isize),
    UnsignedNumber(usize),
    Text(String),
}

pub enum Where {
    Equal(String, ColVal),
    NotEqual(String, ColVal),
    LessThan(String, ColVal),
    LessThanOrEqual(String, ColVal),
    GreaterThan(String, ColVal),
    GreaterThanOrEqual(String, ColVal),
    Between(String, ColVal),
    In(String, ColVal),
    Like(String, ColVal),
    And(Box<Vec<Where>>),
    Or(Box<Vec<Where>>),
}

mod select;
