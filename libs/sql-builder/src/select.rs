use super::Operation;
pub use super::{CommonInstructions, Where, WhereInstructions};
use std::fmt::Display;

/// Controller for building `SELECT` SQL statements(prepared)
pub struct Select(pub(crate) String);

impl Select {
    #[inline(always)]
    pub fn new(columns: Option<Vec<String>>, from: String) -> Self {
        Self(format!("{}", Operation::Select(columns, from)))
    }

    #[inline(always)]
    pub fn new_distinct(columns: Vec<String>, from: String) -> Self {
        Self(format!("{}", Operation::SelectDistinct(columns, from)))
    }

    #[inline(always)]
    pub fn add_arg(&self, arg: SelectArg) -> Self {
        Self(format!("{} {}", self.0, arg))
    }

    #[inline(always)]
    pub fn exists(&self) -> Self {
        Self(format!("SELECT EXISTS({})", self.0))
    }
}

impl CommonInstructions for Select {
    #[inline(always)]
    fn to_string(&self) -> String {
        format!("{};", self.0)
    }
}

impl WhereInstructions for Select {
    #[inline(always)]
    fn open_parentheses(&self) -> Self {
        Self(format!("{} (", self.0))
    }

    #[inline(always)]
    fn close_parentheses(&self) -> Self {
        Self(format!("{} )", self.0))
    }

    #[inline(always)]
    fn and_keyword(&self) -> Self {
        Self(format!("{} AND", self.0))
    }

    #[inline(always)]
    fn or_keyword(&self) -> Self {
        Self(format!("{} OR", self.0))
    }

    fn where_condition(&self, w: Where) -> Self {
        if let Some((_, last)) = self.0.rsplit_once(' ') {
            match last {
                "WHERE" | "(" | "OR" | "AND" => {
                    return Self(format!("{} {}", self.0, w));
                }
                _ => (),
            };
        }

        Self(format!("{} WHERE {}", self.0, w))
    }

    #[inline(always)]
    fn and_where(&self, w: Where) -> Self {
        Self(format!("{} AND {}", self.0, w))
    }

    #[inline(always)]
    fn or_where(&self, w: Where) -> Self {
        Self(format!("{} OR {}", self.0, w))
    }
}

pub enum SelectArg {
    /// 1st arg: Value for "LIMIT"
    Limit(usize),
    /// 1st arg: Value for "OFFSET"
    Offset(usize),
    /// 1st arg: List of columns with order type
    OrderBy(Vec<OrderType>),
    /// 1st arg: Vector of column names
    GroupBy(Vec<String>),
    /// 1st arg: Where condition
    Having(Where),
    /// 1st arg: Table to join
    /// 2nd arg: Column name of joined table (Must include table name, like "table.column_name")
    /// 3rd arg: Column name of FROM's to connect (Must include table name, like "table.column_name")
    InnerJoin(String, String, String),
    /// 1st arg: Table to join
    /// 2nd arg: Column name of joined table (Must include table name, like "table.column_name")
    /// 3rd arg: Column name of FROM's to connect (Must include table name, like "table.column_name")
    LeftJoin(String, String, String),
    /// 1st arg: Table to join
    CrossJoin(String),
    /// 1st arg: Select controller
    Except(Select),
}

pub enum OrderType {
    /// 1st arg: Column name
    Asc(String),
    /// 1st arg: Column name
    Desc(String),
}

impl Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Asc(column) => write!(f, "{} ASC", column),
            OrderType::Desc(column) => write!(f, "{} DESC", column),
        }
    }
}

impl Display for SelectArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Except(sql) => write!(f, "EXCEPT {}", sql.0),

            Self::Limit(limit) => write!(f, "LIMIT {limit}"),

            Self::OrderBy(order_types) => {
                let order_types: Vec<String> = order_types.iter().map(|t| t.to_string()).collect();
                let order_types = order_types.join(", ");
                write!(f, "ORDER BY {order_types}")
            }

            Self::Offset(offset) => write!(f, "OFFSET {offset}"),

            Self::GroupBy(columns) => {
                if columns.is_empty() {
                    writeln!(
                        f,
                        "At least one column must be defined for DISTINCT queries."
                    )?;
                    return Err(std::fmt::Error);
                }

                let columns = columns.join(", ");

                write!(f, "GROUP BY {}", columns)
            }

            Self::Having(condition) => write!(f, "HAVING {}", condition),

            Self::InnerJoin(table, current_table_column, target_table_column) => write!(
                f,
                "INNER JOIN {} ON {} = {}",
                table, current_table_column, target_table_column
            ),

            Self::LeftJoin(table, current_table_column, target_table_column) => write!(
                f,
                "LEFT JOIN {} ON {} = {}",
                table, current_table_column, target_table_column
            ),

            Self::CrossJoin(table) => write!(f, "CROSS JOIN {}", table),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_select() {
        let statement = "SELECT * FROM packages;";
        let sql = Select::new(None, String::from("packages"));
        assert_eq!(statement, sql.to_string());

        let statement = "SELECT id, name, kind FROM packages;";
        let cols = vec![
            String::from("id"),
            String::from("name"),
            String::from("kind"),
        ];
        let sql = Select::new(Some(cols), String::from("packages"));
        assert_eq!(statement, sql.to_string());
    }

    #[test]
    fn test_ifnull_with_max() {
        let statement = String::from("SELECT IFNULL(MAX(index_timestamp), 0) FROM packages;");
        let cols = vec![String::from("IFNULL(MAX(index_timestamp), 0)")];
        let sql = Select::new(Some(cols), String::from("packages"));
        assert_eq!(statement, sql.to_string());
    }

    #[test]
    fn test_select_distinct() {
        let statement = "SELECT DISTINCT name FROM packages;";
        let cols = vec![String::from("name")];
        let sql = Select::new_distinct(cols, String::from("packages"));
        assert_eq!(statement, sql.to_string());
    }

    #[test]
    fn test_select_exists() {
        let statement = "SELECT EXISTS(SELECT * FROM packages);";
        let sql = Select::new(None, String::from("packages")).exists();
        assert_eq!(statement, sql.to_string());
    }

    #[test]
    fn test_select_with_conditions() {
        let expected = "SELECT * FROM people WHERE name = ?1;";
        let sql = Select::new(None, String::from("people"))
            .where_condition(Where::Equal(1, String::from("name")));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people WHERE name != ?1;";
        let sql = Select::new(None, String::from("people"))
            .where_condition(Where::NotEqual(1, String::from("name")));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people WHERE age < ?1;";
        let sql = Select::new(None, String::from("people"))
            .where_condition(Where::LessThan(1, String::from("age")));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people WHERE age <= ?1;";
        let sql = Select::new(None, String::from("people"))
            .where_condition(Where::LessThanOrEqual(1, String::from("age")));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people WHERE age > ?1;";
        let sql = Select::new(None, String::from("people"))
            .where_condition(Where::GreaterThan(1, String::from("age")));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people WHERE age >= ?1;";
        let sql = Select::new(None, String::from("people"))
            .where_condition(Where::GreaterThanOrEqual(1, String::from("age")));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people WHERE age BETWEEN ?1 AND ?2;";
        let sql = Select::new(None, String::from("people")).where_condition(Where::Between(
            1,
            2,
            String::from("age"),
        ));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people WHERE age NOT BETWEEN ?1 AND ?2;";
        let sql = Select::new(None, String::from("people")).where_condition(Where::NotBetween(
            1,
            2,
            String::from("age"),
        ));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people WHERE age IN (?1);";
        let sql = Select::new(None, String::from("people"))
            .where_condition(Where::In(vec![1], String::from("age")));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people WHERE age NOT IN (?1, ?2, ?3, ?4);";
        let sql = Select::new(None, String::from("people"))
            .where_condition(Where::NotIn(vec![1, 2, 3, 4], String::from("age")));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people WHERE name LIKE ?1;";
        let sql = Select::new(None, String::from("people"))
            .where_condition(Where::Like(1, String::from("name")));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people WHERE name NOT LIKE ?1;";
        let sql = Select::new(None, String::from("people"))
            .where_condition(Where::NotLike(1, String::from("name")));
        assert_eq!(expected, sql.to_string());
    }

    #[test]
    fn test_select_with_args() {
        let expected = "SELECT * FROM people LIMIT 100;";
        let sql = Select::new(None, String::from("people")).add_arg(SelectArg::Limit(100));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people OFFSET 100;";
        let sql = Select::new(None, String::from("people")).add_arg(SelectArg::Offset(100));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people ORDER BY size ASC;";
        let sql = Select::new(None, String::from("people")).add_arg(SelectArg::OrderBy(vec![
            OrderType::Asc(String::from("size")),
        ]));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people ORDER BY size DESC;";
        let sql = Select::new(None, String::from("people")).add_arg(SelectArg::OrderBy(vec![
            OrderType::Desc(String::from("size")),
        ]));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people GROUP BY name, size;";
        let columns = vec![String::from("name"), String::from("size")];
        let sql = Select::new(None, String::from("people")).add_arg(SelectArg::GroupBy(columns));
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT * FROM people HAVING size = ?1;";
        let sql = Select::new(None, String::from("people"))
            .add_arg(SelectArg::Having(Where::Equal(1, String::from("size"))));
        assert_eq!(expected, sql.to_string());

        let expected =
            "SELECT id FROM people INNER JOIN employees ON employees.person_id = people.id;";
        let sql = Select::new(Some(vec![String::from("id")]), String::from("people")).add_arg(
            SelectArg::InnerJoin(
                String::from("employees"),
                String::from("employees.person_id"),
                String::from("people.id"),
            ),
        );
        assert_eq!(expected, sql.to_string());

        let expected =
            "SELECT id FROM people LEFT JOIN employees ON employees.person_id = people.id;";
        let sql = Select::new(Some(vec![String::from("id")]), String::from("people")).add_arg(
            SelectArg::LeftJoin(
                String::from("employees"),
                String::from("employees.person_id"),
                String::from("people.id"),
            ),
        );
        assert_eq!(expected, sql.to_string());

        let expected = "SELECT surname FROM people EXCEPT SELECT surname FROM employees;";
        let sql1 = Select::new(
            Some(vec![String::from("surname")]),
            String::from("employees"),
        );
        let sql = Select::new(Some(vec![String::from("surname")]), String::from("people"))
            .add_arg(SelectArg::Except(sql1));
        assert_eq!(expected, sql.to_string());

        let expected = String::from("SELECT v_major, v_minor, v_patch, v_tag, v_readable FROM repository WHERE name = ?1 ORDER BY v_major DESC, v_minor DESC, v_patch DESC LIMIT 1;");
        let sql1 = vec![
            String::from("v_major"),
            String::from("v_minor"),
            String::from("v_patch"),
            String::from("v_tag"),
            String::from("v_readable"),
        ];
        let sql = Select::new(Some(sql1), String::from("repository"))
            .where_condition(Where::Equal(1, String::from("name")))
            .add_arg(SelectArg::OrderBy(vec![
                OrderType::Desc(String::from("v_major")),
                OrderType::Desc(String::from("v_minor")),
                OrderType::Desc(String::from("v_patch")),
            ]))
            .add_arg(SelectArg::Limit(1));
        assert_eq!(expected, sql.to_string());
    }

    #[test]
    fn test_select_with_nested_conditions() {
        let expected = "SELECT last_name, first_name FROM employees WHERE first_name = ?1 OR ( last_name = ?2 AND first_name = ?3 ) OR ( employee_id = ?4 AND last_name = ?5 );";

        let sql = Select::new(
            Some(vec![String::from("last_name"), String::from("first_name")]),
            String::from("employees"),
        )
        .where_condition(Where::Equal(1, String::from("first_name")))
        .or_keyword()
        .open_parentheses()
        .where_condition(Where::Equal(2, String::from("last_name")))
        .and_where(Where::Equal(3, String::from("first_name")))
        .close_parentheses()
        .or_keyword()
        .open_parentheses()
        .where_condition(Where::Equal(4, String::from("employee_id")))
        .and_where(Where::Equal(5, String::from("last_name")))
        .close_parentheses();

        assert_eq!(expected, sql.to_string());
    }
}
