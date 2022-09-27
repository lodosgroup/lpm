use super::*;

struct Select(String);

impl Select {
    pub fn new(columns: Option<Vec<String>>, from: String) -> Self {
        Self(format!("{}", Operation::Select(columns, from)))
    }

    /// Adds '('
    #[inline(always)]
    pub fn open_parentheses(&self) -> Self {
        Self(format!("{} (", self.0))
    }

    /// Adds ')'
    #[inline(always)]
    pub fn close_parentheses(&self) -> Self {
        Self(format!("{} )", self.0))
    }

    /// Only adds 'AND' keyword
    #[inline(always)]
    pub fn and_keyword(&self) -> Self {
        Self(format!("{} AND", self.0))
    }

    /// Only adds 'OR' keyword
    #[inline(always)]
    pub fn or_keyword(&self) -> Self {
        Self(format!("{} OR", self.0))
    }

    /// Adds contiditon
    pub fn where_condition(&self, w: Where) -> Self {
        if let Some((_, last)) = self.0.rsplit_once(" ") {
            match last {
                "WHERE" | "(" | "OR" | "AND" => {
                    return Self(format!("{} {}", self.0, w));
                }
                _ => (),
            };
        }

        Self(format!("{} WHERE {}", self.0, w))
    }

    /// Adds contiditon as 'AND'
    #[inline(always)]
    pub fn and_where(&self, w: Where) -> Self {
        Self(format!("{} AND {}", self.0, w))
    }

    /// Adds contiditon as 'OR'
    #[inline(always)]
    pub fn or_where(&self, w: Where) -> Self {
        Self(format!("{} OR {}", self.0, w))
    }

    /// Returns prepared statement in String form
    #[inline(always)]
    pub fn to_string(&self) -> String {
        format!("{};", self.0)
    }
}

#[test]
#[ignore]
fn select_builder() {
    const PRE_ID_ACTIVE: u8 = 0;
    const PRE_ID_USERNAME: u8 = 1;
    const PRE_ID_AGE: u8 = 2;
    const PRE_ID_RETIRED: u8 = 3;
    const PRE_ID_RETIRED2: u8 = 4;

    let select_cols = vec![
        String::from("id"),
        String::from("name"),
        String::from("surname"),
    ];

    let sql = Select::new(Some(select_cols), String::from("users"));

    let sql = sql
        .where_condition(Where::NotEqual(PRE_ID_ACTIVE, String::from("active")))
        .or_keyword()
        .open_parentheses()
        .where_condition(Where::Equal(PRE_ID_USERNAME, String::from("username")))
        .and_where(Where::GreaterThanOrEqual(PRE_ID_AGE, String::from("age")))
        .and_where(Where::NotEqual(PRE_ID_RETIRED, String::from("retired")))
        .close_parentheses()
        .or_where(Where::NotEqual(PRE_ID_RETIRED2, String::from("retired")));

    println!("{}", sql.to_string());

    assert!(false);
}
