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
