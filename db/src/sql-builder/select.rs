use super::*;

#[derive(Clone)]
struct Select(String);

impl Select {
    pub fn new(columns: Option<Vec<String>>, from: String) -> Self {
        Self(format!("{}", Operation::Select(columns, from)))
    }

    /// Adds '('
    pub fn initialize_sub_statement(&mut self) -> Self {
        self.0 = format!("{} (", self.0);
        self.clone()
    }

    /// Adds ')'
    pub fn end_sub_statement(&mut self) -> Self {
        self.0 = format!("{} )", self.0);
        self.clone()
    }

    /// Only adds 'WHERE' keyword
    pub fn where_keyword(&mut self) -> Self {
        self.0 = format!("{} WHERE", self.0);
        self.clone()
    }

    /// Adds contiditon
    pub fn r#where(&mut self, w: Where) -> Self {
        self.0 = format!("{} WHERE {}", self.0, w);
        self.clone()
    }

    /// Adds contiditon without 'WHERE' keyword
    pub fn sub_where(&mut self, w: Where) -> Self {
        self.0 = format!("{} {}", self.0, w);
        self.clone()
    }

    /// Adds contiditon as 'AND'
    pub fn and_where(&mut self, w: Where) -> Self {
        self.0 = format!("{} AND {}", self.0, w);
        self.clone()
    }

    /// Adds contiditon as 'OR'
    pub fn or_where(&mut self, w: Where) -> Self {
        self.0 = format!("{} OR {}", self.0, w);
        self.clone()
    }

    pub fn statement(&self) -> String {
        format!("{};", self.0.clone())
    }
}

#[test]
#[ignore]
fn select_builder() {
    const PRE_ID_USERNAME: u8 = 0;
    const PRE_ID_AGE: u8 = 1;
    const PRE_ID_RETIRED: u8 = 2;
    const PRE_ID_RETIRED2: u8 = 3;

    let select_cols = vec![
        String::from("id"),
        String::from("name"),
        String::from("surname"),
    ];

    let mut sql = Select::new(Some(select_cols), String::from("users"));

    sql.where_keyword()
        .initialize_sub_statement()
        .sub_where(Where::Equal(PRE_ID_USERNAME, String::from("username")))
        .and_where(Where::GreaterThanOrEqual(PRE_ID_AGE, String::from("age")))
        .and_where(Where::NotEqual(PRE_ID_RETIRED, String::from("retired")))
        .end_sub_statement()
        .or_where(Where::NotEqual(PRE_ID_RETIRED2, String::from("retired")));

    println!("{}", sql.statement());

    assert!(false);
}
