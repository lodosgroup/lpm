pub use super::CommonInstructions;
use super::{select::Select, Operation};

/// Controller for building `INSERT` SQL statements(prepared)
pub struct Insert(String);

pub struct Column(pub(crate) String, pub(crate) u8);

impl Column {
    #[inline(always)]
    pub fn new(name: String, prepared_id: u8) -> Self {
        Self(name, prepared_id)
    }
}

impl Insert {
    /// Inserts default values if first arg is None or has 0 elements
    #[inline(always)]
    pub fn new(columns: Option<Vec<Column>>, into: String) -> Self {
        Self(format!("{}", Operation::Insert(into, columns)))
    }

    #[inline(always)]
    pub fn new_from_select(select: Select, into: String) -> Self {
        Self(format!("{}", Operation::InsertFromSelect(into, select)))
    }

    #[inline(always)]
    pub fn insert_another_row(&self, column_pre_ids: Vec<u8>) -> Self {
        let prepared_ids: Vec<String> =
            column_pre_ids.iter().map(|id| format!("?{}", id)).collect();
        let prepared_ids = prepared_ids.join(", ");

        Self(format!("{}, ({})", self.0, prepared_ids))
    }
}

impl CommonInstructions for Insert {
    #[inline(always)]
    fn to_string(&self) -> String {
        format!("{};", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_with_default_values() {
        let statement = "INSERT INTO packages DEFAULT VALUES;";
        let sql = Insert::new(None, String::from("packages"));
        assert_eq!(statement, sql.to_string());

        let sql = Insert::new(Some(vec![]), String::from("packages"));
        assert_eq!(statement, sql.to_string());
    }

    #[test]
    fn test_insert_with_values() {
        let statement = "INSERT INTO packages (name, kind, size) VALUES(?1, ?2, ?3);";
        let sql = Insert::new(
            Some(vec![
                Column::new(String::from("name"), 1),
                Column::new(String::from("kind"), 2),
                Column::new(String::from("size"), 3),
            ]),
            String::from("packages"),
        );
        assert_eq!(statement, sql.to_string());
    }

    #[test]
    fn test_insert_from_select() {
        let statement = "INSERT INTO packages SELECT id, name, kind FROM packages;";
        let cols = vec![
            String::from("id"),
            String::from("name"),
            String::from("kind"),
        ];
        let select = Select::new(Some(cols), String::from("packages"));
        let sql = Insert::new_from_select(select, String::from("packages"));
        assert_eq!(statement, sql.to_string());
    }
}
