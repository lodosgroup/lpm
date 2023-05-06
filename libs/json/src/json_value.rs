use std::{collections::BTreeMap, ops::Index};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum JsonValue {
    Plain(String),
    Object(Object),
    Array(Vec<JsonValue>),
    #[default]
    Null,
}

impl Default for &JsonValue {
    fn default() -> Self {
        &JsonValue::Null
    }
}

impl Index<usize> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            JsonValue::Plain(_) | JsonValue::Object(_) | JsonValue::Null => &Self::Output::Null,
            JsonValue::Array(array) => array.get(index).unwrap_or_default(),
        }
    }
}

impl Index<&str> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: &str) -> &Self::Output {
        match self {
            JsonValue::Plain(_) | JsonValue::Array(_) | JsonValue::Null => &Self::Output::Null,
            JsonValue::Object(object) => object.get(index).unwrap_or_default(),
        }
    }
}

type Object = BTreeMap<String, JsonValue>;

macro_rules! impl_as_fn {
    ($fn_name: ident, $type: ident) => {
        pub fn $fn_name(&self) -> Option<$type> {
            if let JsonValue::Plain(plain) = self {
                plain.parse().ok()
            } else {
                None
            }
        }
    };
}

impl JsonValue {
    impl_as_fn!(as_bool, bool);
    impl_as_fn!(as_i8, i8);
    impl_as_fn!(as_i6, i16);
    impl_as_fn!(as_i32, i32);
    impl_as_fn!(as_i64, i64);
    impl_as_fn!(as_i128, i128);
    impl_as_fn!(as_isize, isize);

    impl_as_fn!(as_u8, u8);
    impl_as_fn!(as_u16, u16);
    impl_as_fn!(as_u32, u32);
    impl_as_fn!(as_u64, u64);
    impl_as_fn!(as_u128, u128);
    impl_as_fn!(as_usize, usize);

    impl_as_fn!(as_f32, f32);
    impl_as_fn!(as_f64, f64);

    impl_as_fn!(to_string, String);

    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    pub fn is_object(&self) -> bool {
        matches!(self, JsonValue::Object(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, JsonValue::Array(_))
    }
}
