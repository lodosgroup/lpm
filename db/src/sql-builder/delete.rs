use super::{CommonInstructions, Operation, Where, WhereInstructions};
use std::fmt::Display;

pub struct Delete(String);

impl Delete {
    #[inline(always)]
    pub fn new(from: String) -> Self {
        Self(format!("{}", Operation::Delete(from)))
    }

    #[inline(always)]
    pub fn add_arg(&self, arg: DeleteArg) -> Self {
        Self(format!("{} {}", self.0, arg))
    }
}

impl CommonInstructions for Delete {
    #[inline(always)]
    fn to_string(&self) -> String {
        format!("{};", self.0)
    }
}

impl WhereInstructions for Delete {
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

    #[inline(always)]
    fn and_where(&self, w: Where) -> Self {
        Self(format!("{} AND {}", self.0, w))
    }

    #[inline(always)]
    fn or_where(&self, w: Where) -> Self {
        Self(format!("{} OR {}", self.0, w))
    }
}

pub enum DeleteArg {
    Limit(usize),
    Offset(usize),
    OrderByAsc(String),
    OrderByDesc(String),
}

impl Display for DeleteArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Limit(limit) => write!(f, "LIMIT {}", limit),

            Self::OrderByAsc(name) => write!(f, "ORDER BY {} ASC", name),

            Self::OrderByDesc(name) => write!(f, "ORDER BY {} DESC", name),

            Self::Offset(offset) => write!(f, "OFFSET {}", offset),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_delete() {
        let statement = "DELETE FROM packages;";
        let sql = Delete::new(String::from("packages"));
        assert_eq!(statement, sql.to_string());
    }

    #[test]
    fn test_delete_with_conditions() {
        let expected = "DELETE FROM people WHERE name = ?1;";
        let sql = Delete::new(String::from("people"))
            .where_condition(Where::Equal(1, String::from("name")));
        assert_eq!(expected, sql.to_string());

        let expected = "DELETE FROM people WHERE name != ?1;";
        let sql = Delete::new(String::from("people"))
            .where_condition(Where::NotEqual(1, String::from("name")));
        assert_eq!(expected, sql.to_string());

        let expected = "DELETE FROM people WHERE age < ?1;";
        let sql = Delete::new(String::from("people"))
            .where_condition(Where::LessThan(1, String::from("age")));
        assert_eq!(expected, sql.to_string());

        let expected = "DELETE FROM people WHERE age <= ?1;";
        let sql = Delete::new(String::from("people"))
            .where_condition(Where::LessThanOrEqual(1, String::from("age")));
        assert_eq!(expected, sql.to_string());

        let expected = "DELETE FROM people WHERE age > ?1;";
        let sql = Delete::new(String::from("people"))
            .where_condition(Where::GreaterThan(1, String::from("age")));
        assert_eq!(expected, sql.to_string());

        let expected = "DELETE FROM people WHERE age >= ?1;";
        let sql = Delete::new(String::from("people"))
            .where_condition(Where::GreaterThanOrEqual(1, String::from("age")));
        assert_eq!(expected, sql.to_string());

        let expected = "DELETE FROM people WHERE age BETWEEN ?1 AND ?2;";
        let sql = Delete::new(String::from("people")).where_condition(Where::Between(
            1,
            2,
            String::from("age"),
        ));
        assert_eq!(expected, sql.to_string());

        let expected = "DELETE FROM people WHERE age NOT BETWEEN ?1 AND ?2;";
        let sql = Delete::new(String::from("people")).where_condition(Where::NotBetween(
            1,
            2,
            String::from("age"),
        ));
        assert_eq!(expected, sql.to_string());

        let expected = "DELETE FROM people WHERE age IN ?1;";
        let sql =
            Delete::new(String::from("people")).where_condition(Where::In(1, String::from("age")));
        assert_eq!(expected, sql.to_string());

        let expected = "DELETE FROM people WHERE age NOT IN ?1;";
        let sql = Delete::new(String::from("people"))
            .where_condition(Where::NotIn(1, String::from("age")));
        assert_eq!(expected, sql.to_string());

        let expected = "DELETE FROM people WHERE name LIKE ?1;";
        let sql = Delete::new(String::from("people"))
            .where_condition(Where::Like(1, String::from("name")));
        assert_eq!(expected, sql.to_string());

        let expected = "DELETE FROM people WHERE name NOT LIKE ?1;";
        let sql = Delete::new(String::from("people"))
            .where_condition(Where::NotLike(1, String::from("name")));
        assert_eq!(expected, sql.to_string());
    }

    #[test]
    fn test_delete_with_args() {
        let expected = "DELETE FROM people LIMIT 100;";
        let sql = Delete::new(String::from("people")).add_arg(DeleteArg::Limit(100));
        assert_eq!(expected, sql.to_string());

        let expected = "DELETE FROM people OFFSET 100;";
        let sql = Delete::new(String::from("people")).add_arg(DeleteArg::Offset(100));
        assert_eq!(expected, sql.to_string());

        let expected = "DELETE FROM people ORDER BY size ASC;";
        let sql = Delete::new(String::from("people"))
            .add_arg(DeleteArg::OrderByAsc(String::from("size")));
        assert_eq!(expected, sql.to_string());

        let expected = "DELETE FROM people ORDER BY size DESC;";
        let sql = Delete::new(String::from("people"))
            .add_arg(DeleteArg::OrderByDesc(String::from("size")));
        assert_eq!(expected, sql.to_string());
    }

    #[test]
    fn test_delete_with_nested_conditions() {
        let expected = "DELETE FROM employees WHERE first_name = ?1 OR ( last_name = ?2 AND first_name = ?3 ) OR ( employee_id = ?4 AND last_name = ?5 );";

        let sql = Delete::new(String::from("employees"))
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
