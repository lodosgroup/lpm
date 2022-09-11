pub struct SqlBuilder {}

pub enum Operation {
    // 1st arg: Vector of column names. None means "*".
    // 2nd arg: Arg for "FROM".
    Select(Option<Vec<String>>, String),
    Create(String, Option<CreateOperationArg>),
    Update(String),
    Delete(String),
    Replace(String),
    Insert(String),
}

pub enum CreateOperationArg {
    IfNotExists,
}

mod select;
