use super::{Column, CommonInstructions, Operation, Where, WhereInstructions};

/// Controller for building `UPDATE` SQL statements(prepared)
pub struct Update(String);

impl Update {
    #[inline(always)]
    pub fn new(columns: Vec<Column>, into: String) -> Self {
        Self(format!("{}", Operation::Update(into, columns)))
    }
}

impl CommonInstructions for Update {
    #[inline(always)]
    fn to_string(&self) -> String {
        format!("{};", self.0)
    }
}

impl WhereInstructions for Update {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update() {
        let statement = "UPDATE packages SET name = ?1, kind = ?2, size = ?3;";
        let sql = Update::new(
            vec![
                Column::new(String::from("name"), 1),
                Column::new(String::from("kind"), 2),
                Column::new(String::from("size"), 3),
            ],
            String::from("packages"),
        );

        assert_eq!(statement, sql.to_string());
    }

    #[test]
    fn test_update_with_where_condition() {
        let statement = "UPDATE packages SET name = ?1, kind = ?2, size = ?3 WHERE id = ?1;";
        let sql = Update::new(
            vec![
                Column::new(String::from("name"), 1),
                Column::new(String::from("kind"), 2),
                Column::new(String::from("size"), 3),
            ],
            String::from("packages"),
        )
        .where_condition(Where::Equal(1, String::from("id")));

        assert_eq!(statement, sql.to_string());
    }
}
