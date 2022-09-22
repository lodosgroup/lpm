use std::fmt::Display;

pub struct SqlBuilder {
    pub operation: Operation,
    pub criteria: Where,
    // TODO
    // Order By
    // Limit
    // etc
}

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
                    Some(t) if !t.is_empty() => t.join(", "),
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

pub enum Where {
    Criteria(WhereOperation),
}

impl Display for Where {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Where::Criteria(where_ops) => {
                let condition = match where_ops {
                    WhereOperation::Equal(index, name) => format!("{} = ?{}", name, index),

                    WhereOperation::NotEqual(index, name) => format!("{} != ?{}", name, index),

                    WhereOperation::LessThan(index, name) => format!("{} < ?{}", name, index),

                    WhereOperation::LessThanOrEqual(index, name) => {
                        format!("{} <= ?{}", name, index)
                    }

                    WhereOperation::GreaterThan(index, name) => format!("{} > ?{}", name, index),

                    WhereOperation::GreaterThanOrEqual(index, name) => {
                        format!("{} >= ?{}", name, index)
                    }

                    WhereOperation::Between(index1, index2, name) => {
                        format!("{} BETWEEN ?{} AND ?{}", name, index1, index2)
                    }

                    WhereOperation::NotBetween(index1, index2, name) => {
                        format!("{} NOT BETWEEN ?{} AND ?{}", name, index1, index2)
                    }

                    WhereOperation::In(index, name) => {
                        format!("{} IN ?{}", name, index)
                    }

                    WhereOperation::NotIn(index, name) => {
                        format!("{} NOT IN ?{}", name, index)
                    }

                    WhereOperation::Like(index, name) => {
                        format!("{} LIKE ?{}", name, index)
                    }

                    WhereOperation::NotLike(index, name) => {
                        format!("{} NOT LIKE ?{}", name, index)
                    }

                    WhereOperation::And(_) => todo!(),
                    WhereOperation::Or(_) => todo!(),
                };

                write!(f, "WHERE {}", condition)
            }
        }
    }
}

/// Column's index to bind value following with it's name
pub enum WhereOperation {
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
    And(Box<Vec<WhereOperation>>),
    Or(Box<Vec<WhereOperation>>),
}

mod select;

#[test]
#[ignore]
fn custom_test() {
    let sb = SqlBuilder {
        operation: Operation::Select(
            Some(vec![
                String::from("id"),
                String::from("name"),
                String::from("surname"),
            ]),
            "users".to_string(),
        ),
        criteria: Where::Criteria(WhereOperation::Equal(1, "id".to_string())),
    };
    println!("{} {}", sb.operation, sb.criteria);
    assert!(false);
}
