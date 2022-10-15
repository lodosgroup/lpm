#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use std::collections::BTreeMap;
use std::fmt::Display;

const OBJECT_OPENER: char = '{';
const OBJECT_CLOSER: char = '}';
const ARRAY_OPENER: char = '[';
const ARRAY_CLOSER: char = ']';

const DOUBLE_QUOTE: char = '"';
const COMMA: char = ',';
const COLON: char = ':';
const WHITESPACE: char = ' ';
const ESCAPE: char = '\\';
const NEWLINE: char = '\n';

pub struct Json<'a>(&'a str);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonValue {
    Plain(String),
    Object(Object),
    Array(Vec<JsonValue>),
}

type Object = BTreeMap<String, JsonValue>;

macro_rules! impl_as_fn {
    ($fn_name: ident, $type: ident) => {
        fn $fn_name(&self) -> Option<$type> {
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
}

impl Display for Json<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Json({})", self.0)
    }
}

impl std::fmt::Debug for Json<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Json({})", self.0)
    }
}

impl<'a> Json<'a> {
    #[inline(always)]
    pub fn new(json_string: &'a str) -> Self {
        Self(json_string)
    }

    fn check_brackets(openers: &Vec<char>, closers: &Vec<char>) -> bool {
        if openers.len() != closers.len() {
            return false;
        }

        for (index, c) in openers.iter().enumerate() {
            match *c {
                OBJECT_OPENER => {
                    if closers[index] != OBJECT_CLOSER {
                        return false;
                    }
                }

                ARRAY_OPENER => {
                    if closers[index] != ARRAY_CLOSER {
                        return false;
                    }
                }
                _ => {}
            }
        }

        true
    }

    // TODO
    fn iterate_tokens(
        chars: &mut Vec<char>,
        root_object: &mut JsonValue,
        outer_key: Option<String>,
        mut inner_data: Option<&mut JsonValue>,
    ) {
        let ptr: *const char = chars.as_ptr();

        let mut openers: Vec<char> = vec![];
        let mut closers: Vec<char> = vec![];

        // Controller value for when to parse key/value
        let pass_the_key = outer_key.is_some();
        let mut is_key = false;
        let mut is_string_value = false;
        let mut is_non_string_value = false;

        let mut key = outer_key.unwrap_or_default();
        let mut value = String::default();

        // Do the iteration via raw pointer to move back-forth without cloning things
        let mut pointer_cursor = 0;
        let mut chars_len = chars.len();

        while pointer_cursor < chars_len {
            #[allow(unsafe_code)]
            let previous_char = || {
                if pointer_cursor != 0 {
                    unsafe { Some(*ptr.add(pointer_cursor - 1)) }
                } else {
                    None
                }
            };

            #[allow(unsafe_code)]
            let next_char = || {
                if pointer_cursor + 1 < chars_len {
                    unsafe { Some(*ptr.add(pointer_cursor + 1)) }
                } else {
                    None
                }
            };

            #[allow(unsafe_code)]
            let current_char = unsafe { *ptr.add(pointer_cursor) };

            // println!(
            //     "current_char: {}, pass_the_key: {}, is_key: {}, is_string_value: {}, is_non_string_value: {}",
            //     current_char, pass_the_key, is_key, is_string_value, is_non_string_value
            // );

            // println!("key {:?}", &key);
            // println!("value {:?}", &value);

            // Parse number and boolean values
            match current_char {
                OBJECT_OPENER if inner_data.is_none() && openers.is_empty() => {
                    *root_object = JsonValue::Object(BTreeMap::default());
                    openers.push(current_char);
                }

                ARRAY_OPENER if inner_data.is_none() && openers.is_empty() => {
                    *root_object = JsonValue::Array(vec![]);
                    openers.push(current_char);
                }

                // Ignore out-tree spaces/new lines
                NEWLINE | WHITESPACE if !is_key && !is_string_value => {
                    chars.remove(pointer_cursor);
                    chars_len -= 1;
                    continue;
                }

                // TODO
                // parse array/objects
                // Move into inner node
                ARRAY_OPENER if !is_string_value => {
                    openers.push(current_char);

                    chars.drain(0..pointer_cursor + 1);
                    pointer_cursor = 0;

                    let mut local_inner = JsonValue::Array(vec![]);
                    let mut local_inner = if let Some(inner) = &mut inner_data {
                        if let JsonValue::Array(i) = inner {
                            inner.clone()
                        } else {
                            local_inner
                        }
                    } else {
                        local_inner
                    };

                    Self::iterate_tokens(
                        chars,
                        root_object,
                        Some(key.clone()),
                        Some(&mut local_inner),
                    );

                    if let Some(JsonValue::Object(inner_object)) = &mut inner_data {
                        inner_object.insert(key.clone(), local_inner);
                    } else {
                        match root_object {
                            JsonValue::Plain(_) => {
                                // TODO
                                // Error here
                            }
                            JsonValue::Object(object) => {
                                //
                                object.insert(key.clone(), local_inner);
                            }
                            JsonValue::Array(array) => {
                                //
                                array.push(local_inner);
                            }
                        };
                    };

                    if !pass_the_key {
                        is_key = false;
                        key = String::default();
                    }

                    is_non_string_value = false;
                    is_string_value = false;
                    value = String::default();

                    chars_len = chars.len();
                }

                OBJECT_OPENER if !is_string_value => {
                    openers.push(current_char);

                    chars.drain(0..pointer_cursor + 1);
                    pointer_cursor = 0;

                    let mut local_inner = JsonValue::Object(BTreeMap::new());
                    Self::iterate_tokens(chars, root_object, None, Some(&mut local_inner));

                    match &mut inner_data {
                        Some(JsonValue::Array(inner_array)) => {
                            inner_array.push(local_inner);
                        }
                        // Burasi neden yukaridan once execute oluyor obje arraylerinde ?
                        Some(JsonValue::Object(inner_object)) => {
                            inner_object.insert(key.clone(), local_inner);
                        }
                        _ => {
                            match root_object {
                                JsonValue::Plain(_) => {
                                    // TODO
                                    // Error here
                                }
                                JsonValue::Object(object) => {
                                    //
                                    object.insert(key.clone(), local_inner);
                                }
                                JsonValue::Array(array) => {
                                    //
                                    array.push(local_inner);
                                }
                            };
                        }
                    };

                    if !pass_the_key {
                        is_key = false;
                        key = String::default();
                    }

                    is_non_string_value = false;
                    is_string_value = false;
                    value = String::default();

                    chars_len = chars.len();
                }

                // TODO
                // parse array/objects
                // Move into upper node
                // THE FUCKING BUG WAS HERE !!!!!!
                OBJECT_CLOSER | ARRAY_CLOSER if !is_string_value => {
                    closers.push(current_char);

                    // If last iteration built a value, then save it
                    if is_non_string_value || is_string_value {
                        match &mut inner_data {
                            Some(JsonValue::Array(array)) => {
                                array.push(JsonValue::Plain(value.clone()));
                            }
                            Some(JsonValue::Object(object)) => {
                                object.insert(key, JsonValue::Plain(value.clone()));
                            }
                            _ => {
                                // TODO
                                // Error HERE
                            }
                        };
                    }

                    // Consume used chars
                    chars.drain(0..pointer_cursor);

                    return;
                }

                // ** only for keys **
                // If current char is opener quote of key
                DOUBLE_QUOTE if !is_key && !is_string_value && previous_char() != Some(COLON) => {
                    is_key = true
                }

                // ** only for keys **
                // If current char is closer quote of key
                DOUBLE_QUOTE if is_key && !is_string_value => is_key = false,

                // ** only for string values **
                // If current char is opener quote of value
                DOUBLE_QUOTE
                    if !is_key
                        && !is_string_value
                        && !is_non_string_value
                        && previous_char() == Some(COLON) =>
                {
                    is_string_value = true
                }

                // ** only for string values **
                // If current char is closer quote of value
                DOUBLE_QUOTE if !is_key && is_string_value && previous_char() != Some(ESCAPE) => {
                    if let Some(JsonValue::Object(inner_object)) = &mut inner_data {
                        inner_object.insert(key.clone(), JsonValue::Plain(value));
                    } else {
                        match root_object {
                            JsonValue::Plain(_) => {
                                // TODO
                                // Error here
                            }
                            JsonValue::Object(object) => {
                                //
                                object.insert(key.clone(), JsonValue::Plain(value));
                            }
                            JsonValue::Array(array) => {
                                //
                                array.push(JsonValue::Plain(value));
                            }
                        };
                    };

                    is_key = false;
                    is_string_value = false;
                    key = String::default();
                    value = String::default();
                }

                non_string_value_char
                    if pass_the_key && !is_string_value && current_char != COMMA =>
                {
                    is_non_string_value = true;
                    value.push(non_string_value_char);
                }

                // ** only for non-string values **
                // If current char the first char of the value
                #[allow(unsafe_code)]
                non_string_value_char
                    if !is_key
                        && !is_string_value
                        && previous_char() == Some(COLON)
                        // checking previous char of previous char if it's double quote
                        // so we can know current char is the first char of the value
                        && unsafe { *ptr.add(pointer_cursor - 2) == DOUBLE_QUOTE } =>
                {
                    is_non_string_value = true;
                    value.push(non_string_value_char);
                }

                // TODO
                // Last item can not be catched because it doesn't have an comma at the end
                COMMA if is_non_string_value && pass_the_key => {
                    match &mut inner_data {
                        Some(JsonValue::Array(array)) => {
                            array.push(JsonValue::Plain(value.clone()));
                        }
                        Some(JsonValue::Object(object)) => {
                            object.insert(key.clone(), JsonValue::Plain(value.clone()));
                        }
                        _ => {
                            // TODO
                            // Error HERE
                        }
                    };
                    is_non_string_value = false;
                    value = String::default();
                }

                // ** only for non-string values **
                // if current char is comma but it's not a value of string
                COMMA if is_non_string_value && !is_key && !is_string_value => {
                    if let Some(JsonValue::Object(inner_object)) = &mut inner_data {
                        inner_object.insert(key.clone(), JsonValue::Plain(value));
                    } else {
                        match root_object {
                            JsonValue::Plain(_) => {
                                // TODO
                                // Error here
                            }
                            JsonValue::Object(object) => {
                                //
                                object.insert(key.clone(), JsonValue::Plain(value));
                            }
                            JsonValue::Array(array) => {
                                //
                                array.push(JsonValue::Plain(value));
                            }
                        };
                    };

                    is_key = false;
                    is_non_string_value = false;
                    key = String::default();
                    value = String::default();
                }

                // Ignore value seperator(comma) when we are not in any key/value stage
                COMMA if !is_key && !is_string_value && !is_non_string_value => {}

                // ** only for keys **
                // If char is part of the key
                c if is_key && !is_string_value && !is_non_string_value && !pass_the_key => {
                    key.push(c)
                }

                // ** only for object and array values **
                // TODO
                c if pass_the_key => value.push(c),

                // ** only for non-string values **
                // If char is part of the non-string value
                c if !is_key && !is_string_value && is_non_string_value => {
                    value.push(c);

                    if let Some(cc) = next_char() {
                        if [OBJECT_CLOSER, ARRAY_CLOSER, WHITESPACE, NEWLINE].contains(&cc) {
                            if let Some(JsonValue::Object(inner_object)) = &mut inner_data {
                                inner_object.insert(key.clone(), JsonValue::Plain(value));
                            } else {
                                match root_object {
                                    JsonValue::Plain(_) => {
                                        // TODO
                                        // Error here
                                    }
                                    JsonValue::Object(object) => {
                                        //
                                        object.insert(key.clone(), JsonValue::Plain(value));
                                    }
                                    JsonValue::Array(array) => {
                                        //
                                        array.push(JsonValue::Plain(value));
                                    }
                                };
                            };

                            is_key = false;
                            is_non_string_value = false;
                            key = String::default();
                            value = String::default();
                        }
                    }
                }

                // If char is part of the value
                c if is_string_value && !is_key && !is_non_string_value => value.push(c),

                _ => {
                    // TODO
                    // Error HERE
                }
            };
            // println!("{:?}", key);
            // println!("{:?}", value);

            pointer_cursor += 1;
        }
    }

    // TODO
    // Map value type should be an enum that wraps following
    // string, number, boolean, object, array types
    fn parse(&self) -> JsonValue {
        let mut chars: Vec<char> = self.0.chars().collect();

        let mut object: BTreeMap<String, JsonValue> = BTreeMap::new();
        let mut root_object = JsonValue::Object(BTreeMap::default());
        Self::iterate_tokens(&mut chars, &mut root_object, None, None);

        root_object
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn measure_time_complexity() {
        let json_content = r#"
            {
              "squadName": "Some value with \"what\"",
              "squadName2": "Some value with\n",
              "deneme1": 89,
              "deneme2": 0,
              "deneme3": -128,
              "deneme4": 50.12,
              "deneme5": -61.5,
              "homeTown": "Metro City",
              "formed": 2016,
              "secretBase": "Super tower",
              "active": true
            }
        "#;

        let now = std::time::Instant::now();
        serde_json::from_str::<serde_json::Value>(json_content).unwrap();
        println!("Serde json elapsed: {:.2?}", now.elapsed());

        let now = std::time::Instant::now();
        Json::new(json_content).parse();
        println!("Lpm json elapsed: {:.2?}", now.elapsed());

        assert!(false, "Failed intentionally.");
    }

    #[test]
    fn test_deserialization_on_json_samples() {
        // Object sample
        {
            let json_content = r#"{
                "name": "Onur",
                "surname": "Ozkan",
                "gender": "male",
                "website": "https://onurozkan.dev",
                "private_account": true,
                "frozen_account": false,
                "other_accounts": {
                    "twitter": "onurozkan_dev",
                    "linkedin": "onurozkandev",
                    "github": "ozkanonur"
                },
                "pets": [
                    {
                        "name": "boncuk",
                        "species": "cat",
                        "fav_foods": ["corn", "polenta"]
                    },
                    {
                        "name": "paskal",
                        "species": "dog",
                        "fav_foods": ["meat", "chicken", "beef"]
                    },
                    {
                        "name": "cesur",
                        "species": "dog",
                        "fav_foods": ["meat", "cheese", "chicken", "beef"]
                    }
                ]
            }"#;

            let mut root_level = BTreeMap::new();

            let mut other_accounts = BTreeMap::new();
            other_accounts.insert(
                "twitter".to_string(),
                JsonValue::Plain("onurozkan_dev".to_string()),
            );
            other_accounts.insert(
                "linkedin".to_string(),
                JsonValue::Plain("onurozkandev".to_string()),
            );
            other_accounts.insert(
                "github".to_string(),
                JsonValue::Plain("ozkanonur".to_string()),
            );

            let mut pets = vec![];

            let mut pet = BTreeMap::new();
            let mut fav_foods = vec![];
            fav_foods.push(JsonValue::Plain("corn".to_string()));
            fav_foods.push(JsonValue::Plain("polenta".to_string()));

            pet.insert("name".to_string(), JsonValue::Plain("boncuk".to_string()));
            pet.insert("species".to_string(), JsonValue::Plain("cat".to_string()));
            pet.insert("fav_foods".to_string(), JsonValue::Array(fav_foods));

            pets.push(JsonValue::Object(pet));

            let mut pet = BTreeMap::new();
            let mut fav_foods = vec![];
            fav_foods.push(JsonValue::Plain("meat".to_string()));
            fav_foods.push(JsonValue::Plain("chicken".to_string()));
            fav_foods.push(JsonValue::Plain("beef".to_string()));

            pet.insert("name".to_string(), JsonValue::Plain("paskal".to_string()));
            pet.insert("species".to_string(), JsonValue::Plain("dog".to_string()));
            pet.insert("fav_foods".to_string(), JsonValue::Array(fav_foods));

            pets.push(JsonValue::Object(pet));

            let mut pet = BTreeMap::new();
            let mut fav_foods = vec![];
            fav_foods.push(JsonValue::Plain("meat".to_string()));
            fav_foods.push(JsonValue::Plain("cheese".to_string()));
            fav_foods.push(JsonValue::Plain("chicken".to_string()));
            fav_foods.push(JsonValue::Plain("beef".to_string()));

            pet.insert("name".to_string(), JsonValue::Plain("cesur".to_string()));
            pet.insert("species".to_string(), JsonValue::Plain("dog".to_string()));
            pet.insert("fav_foods".to_string(), JsonValue::Array(fav_foods));

            pets.push(JsonValue::Object(pet));

            root_level.insert("name".to_string(), JsonValue::Plain("Onur".to_string()));
            root_level.insert("surname".to_string(), JsonValue::Plain("Ozkan".to_string()));
            root_level.insert("gender".to_string(), JsonValue::Plain("male".to_string()));
            root_level.insert(
                "website".to_string(),
                JsonValue::Plain("https://onurozkan.dev".to_string()),
            );
            root_level.insert(
                "private_account".to_string(),
                JsonValue::Plain("true".to_string()),
            );
            root_level.insert(
                "frozen_account".to_string(),
                JsonValue::Plain("false".to_string()),
            );
            root_level.insert(
                "other_accounts".to_string(),
                JsonValue::Object(other_accounts),
            );

            root_level.insert("pets".to_string(), JsonValue::Array(pets));

            let expected_object = JsonValue::Object(root_level);

            let json = Json::new(&json_content);
            let actual_object = json.parse();

            assert_eq!(expected_object, actual_object);
        }

        // Array sample
        {
            let json_content = r#"
            [
                {
                    "id": "feaaf6b1-c637-4243-9570-c97fbaa6620c",
                    "first_name": "Hannie",
                    "last_name": "Bazire",
                    "email": "hbazire0@squarespace.com",
                    "gender": "Female",
                    "private_account": true,
                    "frozen_account": false,
                    "ip_address": "26.229.254.252",
                    "birth": {
                        "day": 01,
                        "month": 05,
                        "year": 1985
                    }
                },
                {
                    "id": "395605bb-bc3b-4bec-9bd2-82ab7d84825e",
                    "first_name": "Cornela",
                    "last_name": "Beckitt",
                    "email": "cbeckitt1@walmart.com",
                    "gender": "Female",
                    "private_account": true,
                    "frozen_account": false,
                    "ip_address": "236.125.222.140",
                    "birth": {
                        "day": 06,
                        "month": 11,
                        "year": 1995
                    }
                },
                {
                    "id": "0fa010a6-9c1d-487b-a69d-767775ae333f",
                    "first_name": "Dougy",
                    "last_name": "Filipowicz",
                    "email": "dfilipowicz2@cbslocal.com",
                    "gender": "Genderqueer",
                    "private_account": true,
                    "frozen_account": false,
                    "ip_address": "207.125.229.204",
                    "birth": {
                        "day": 23,
                        "month": 12,
                        "year": 1969
                    }
                },
                {
                    "id": "2a94416e-39a9-4322-b3d5-c4482e29b36f",
                    "first_name": "Brose",
                    "last_name": "Machel",
                    "email": "bmachel3@ftc.gov",
                    "gender": "Male",
                    "private_account": true,
                    "frozen_account": false,
                    "ip_address": "31.222.19.35",
                    "birth": {
                        "day": 14,
                        "month": 04,
                        "year": 1991
                    }
                },
                {
                    "id": "67a650cf-dabb-4f87-9c70-5719f402cd98",
                    "first_name": "Beverly",
                    "last_name": "Maudling",
                    "email": "bmaudling4@hostgator.com",
                    "gender": "Female",
                    "private_account": true,
                    "frozen_account": false,
                    "ip_address": "128.203.51.91",
                    "birth": {
                        "day": 07,
                        "month": 06,
                        "year": 1988
                    }
                },
                {
                    "id": "2479dbc5-fc0f-4623-9880-f80a52a60e3a",
                    "first_name": "Wait",
                    "last_name": "Behan",
                    "email": "wbehan5@smh.com.au",
                    "gender": "Male",
                    "private_account": true,
                    "frozen_account": false,
                    "ip_address": "217.241.113.180",
                    "birth": {
                        "day": 12,
                        "month": 02,
                        "year": 2001
                    }
                },
                {
                    "id": "77e5906b-9273-42ed-89b4-66b6fc96257b",
                    "first_name": "Regen",
                    "last_name": "de Mullett",
                    "email": "rdemullett6@aboutads.info",
                    "gender": "Male",
                    "private_account": true,
                    "frozen_account": false,
                    "ip_address": "118.165.112.11",
                    "birth": {
                        "day": 22,
                        "month": 09,
                        "year": 1993
                    }
                }
            ]"#;

            let mut root_level = vec![];

            {
                let mut account = BTreeMap::new();
                account.insert(
                    "id".to_string(),
                    JsonValue::Plain("feaaf6b1-c637-4243-9570-c97fbaa6620c".to_string()),
                );
                account.insert(
                    "first_name".to_string(),
                    JsonValue::Plain("Hannie".to_string()),
                );
                account.insert(
                    "last_name".to_string(),
                    JsonValue::Plain("Bazire".to_string()),
                );
                account.insert(
                    "email".to_string(),
                    JsonValue::Plain("hbazire0@squarespace.com".to_string()),
                );
                account.insert("gender".to_string(), JsonValue::Plain("Female".to_string()));
                account.insert(
                    "private_account".to_string(),
                    JsonValue::Plain("true".to_string()),
                );
                account.insert(
                    "frozen_account".to_string(),
                    JsonValue::Plain("false".to_string()),
                );
                account.insert(
                    "ip_address".to_string(),
                    JsonValue::Plain("26.229.254.252".to_string()),
                );

                let mut birth = BTreeMap::new();
                birth.insert("day".to_string(), JsonValue::Plain("01".to_string()));
                birth.insert("month".to_string(), JsonValue::Plain("05".to_string()));
                birth.insert("year".to_string(), JsonValue::Plain("1985".to_string()));

                account.insert("birth".to_string(), JsonValue::Object(birth));

                root_level.push(JsonValue::Object(account));
            }

            {
                let mut account = BTreeMap::new();
                account.insert(
                    "id".to_string(),
                    JsonValue::Plain("395605bb-bc3b-4bec-9bd2-82ab7d84825e".to_string()),
                );
                account.insert(
                    "first_name".to_string(),
                    JsonValue::Plain("Cornela".to_string()),
                );
                account.insert(
                    "last_name".to_string(),
                    JsonValue::Plain("Beckitt".to_string()),
                );
                account.insert(
                    "email".to_string(),
                    JsonValue::Plain("cbeckitt1@walmart.com".to_string()),
                );
                account.insert("gender".to_string(), JsonValue::Plain("Female".to_string()));
                account.insert(
                    "private_account".to_string(),
                    JsonValue::Plain("true".to_string()),
                );
                account.insert(
                    "frozen_account".to_string(),
                    JsonValue::Plain("false".to_string()),
                );
                account.insert(
                    "ip_address".to_string(),
                    JsonValue::Plain("236.125.222.140".to_string()),
                );

                let mut birth = BTreeMap::new();
                birth.insert("day".to_string(), JsonValue::Plain("06".to_string()));
                birth.insert("month".to_string(), JsonValue::Plain("11".to_string()));
                birth.insert("year".to_string(), JsonValue::Plain("1995".to_string()));

                account.insert("birth".to_string(), JsonValue::Object(birth));

                root_level.push(JsonValue::Object(account));
            }

            {
                let mut account = BTreeMap::new();
                account.insert(
                    "id".to_string(),
                    JsonValue::Plain("0fa010a6-9c1d-487b-a69d-767775ae333f".to_string()),
                );
                account.insert(
                    "first_name".to_string(),
                    JsonValue::Plain("Dougy".to_string()),
                );
                account.insert(
                    "last_name".to_string(),
                    JsonValue::Plain("Filipowicz".to_string()),
                );
                account.insert(
                    "email".to_string(),
                    JsonValue::Plain("dfilipowicz2@cbslocal.com".to_string()),
                );
                account.insert(
                    "gender".to_string(),
                    JsonValue::Plain("Genderqueer".to_string()),
                );
                account.insert(
                    "private_account".to_string(),
                    JsonValue::Plain("true".to_string()),
                );
                account.insert(
                    "frozen_account".to_string(),
                    JsonValue::Plain("false".to_string()),
                );
                account.insert(
                    "ip_address".to_string(),
                    JsonValue::Plain("207.125.229.204".to_string()),
                );

                let mut birth = BTreeMap::new();
                birth.insert("day".to_string(), JsonValue::Plain("23".to_string()));
                birth.insert("month".to_string(), JsonValue::Plain("12".to_string()));
                birth.insert("year".to_string(), JsonValue::Plain("1969".to_string()));

                account.insert("birth".to_string(), JsonValue::Object(birth));

                root_level.push(JsonValue::Object(account));
            }

            {
                let mut account = BTreeMap::new();
                account.insert(
                    "id".to_string(),
                    JsonValue::Plain("2a94416e-39a9-4322-b3d5-c4482e29b36f".to_string()),
                );
                account.insert(
                    "first_name".to_string(),
                    JsonValue::Plain("Brose".to_string()),
                );
                account.insert(
                    "last_name".to_string(),
                    JsonValue::Plain("Machel".to_string()),
                );
                account.insert(
                    "email".to_string(),
                    JsonValue::Plain("bmachel3@ftc.gov".to_string()),
                );
                account.insert("gender".to_string(), JsonValue::Plain("Male".to_string()));
                account.insert(
                    "private_account".to_string(),
                    JsonValue::Plain("true".to_string()),
                );
                account.insert(
                    "frozen_account".to_string(),
                    JsonValue::Plain("false".to_string()),
                );
                account.insert(
                    "ip_address".to_string(),
                    JsonValue::Plain("31.222.19.35".to_string()),
                );

                let mut birth = BTreeMap::new();
                birth.insert("day".to_string(), JsonValue::Plain("14".to_string()));
                birth.insert("month".to_string(), JsonValue::Plain("04".to_string()));
                birth.insert("year".to_string(), JsonValue::Plain("1991".to_string()));

                account.insert("birth".to_string(), JsonValue::Object(birth));

                root_level.push(JsonValue::Object(account));
            }

            {
                let mut account = BTreeMap::new();
                account.insert(
                    "id".to_string(),
                    JsonValue::Plain("67a650cf-dabb-4f87-9c70-5719f402cd98".to_string()),
                );
                account.insert(
                    "first_name".to_string(),
                    JsonValue::Plain("Beverly".to_string()),
                );
                account.insert(
                    "last_name".to_string(),
                    JsonValue::Plain("Maudling".to_string()),
                );
                account.insert(
                    "email".to_string(),
                    JsonValue::Plain("bmaudling4@hostgator.com".to_string()),
                );
                account.insert("gender".to_string(), JsonValue::Plain("Female".to_string()));
                account.insert(
                    "private_account".to_string(),
                    JsonValue::Plain("true".to_string()),
                );
                account.insert(
                    "frozen_account".to_string(),
                    JsonValue::Plain("false".to_string()),
                );
                account.insert(
                    "ip_address".to_string(),
                    JsonValue::Plain("128.203.51.91".to_string()),
                );

                let mut birth = BTreeMap::new();
                birth.insert("day".to_string(), JsonValue::Plain("07".to_string()));
                birth.insert("month".to_string(), JsonValue::Plain("06".to_string()));
                birth.insert("year".to_string(), JsonValue::Plain("1988".to_string()));

                account.insert("birth".to_string(), JsonValue::Object(birth));

                root_level.push(JsonValue::Object(account));
            }

            {
                let mut account = BTreeMap::new();
                account.insert(
                    "id".to_string(),
                    JsonValue::Plain("2479dbc5-fc0f-4623-9880-f80a52a60e3a".to_string()),
                );
                account.insert(
                    "first_name".to_string(),
                    JsonValue::Plain("Wait".to_string()),
                );
                account.insert(
                    "last_name".to_string(),
                    JsonValue::Plain("Behan".to_string()),
                );
                account.insert(
                    "email".to_string(),
                    JsonValue::Plain("wbehan5@smh.com.au".to_string()),
                );
                account.insert("gender".to_string(), JsonValue::Plain("Male".to_string()));
                account.insert(
                    "private_account".to_string(),
                    JsonValue::Plain("true".to_string()),
                );
                account.insert(
                    "frozen_account".to_string(),
                    JsonValue::Plain("false".to_string()),
                );
                account.insert(
                    "ip_address".to_string(),
                    JsonValue::Plain("217.241.113.180".to_string()),
                );

                let mut birth = BTreeMap::new();
                birth.insert("day".to_string(), JsonValue::Plain("12".to_string()));
                birth.insert("month".to_string(), JsonValue::Plain("02".to_string()));
                birth.insert("year".to_string(), JsonValue::Plain("2001".to_string()));

                account.insert("birth".to_string(), JsonValue::Object(birth));

                root_level.push(JsonValue::Object(account));
            }

            {
                let mut account = BTreeMap::new();
                account.insert(
                    "id".to_string(),
                    JsonValue::Plain("77e5906b-9273-42ed-89b4-66b6fc96257b".to_string()),
                );
                account.insert(
                    "first_name".to_string(),
                    JsonValue::Plain("Regen".to_string()),
                );
                account.insert(
                    "last_name".to_string(),
                    JsonValue::Plain("de Mullett".to_string()),
                );
                account.insert(
                    "email".to_string(),
                    JsonValue::Plain("rdemullett6@aboutads.info".to_string()),
                );
                account.insert("gender".to_string(), JsonValue::Plain("Male".to_string()));
                account.insert(
                    "private_account".to_string(),
                    JsonValue::Plain("true".to_string()),
                );
                account.insert(
                    "frozen_account".to_string(),
                    JsonValue::Plain("false".to_string()),
                );
                account.insert(
                    "ip_address".to_string(),
                    JsonValue::Plain("118.165.112.11".to_string()),
                );

                let mut birth = BTreeMap::new();
                birth.insert("day".to_string(), JsonValue::Plain("22".to_string()));
                birth.insert("month".to_string(), JsonValue::Plain("09".to_string()));
                birth.insert("year".to_string(), JsonValue::Plain("1993".to_string()));

                account.insert("birth".to_string(), JsonValue::Object(birth));

                root_level.push(JsonValue::Object(account));
            }

            let expected_object = JsonValue::Array(root_level);

            let json = Json::new(&json_content);
            let actual_object = json.parse();

            assert_eq!(expected_object, actual_object);
        }

        // Another object(some more complex) sample
        {
            let json_content = r#"{
                "president": [
                    {
                        "let": 1974033217,
                        "nine": 1043848105,
                        "differ": false,
                        "animal": 1083488261,
                        "section": "pale",
                        "table": "fox"
                    },
                    false,
                    "movie",
                    "bee",
                    true,
                    -58333673.273878574
                ],
                "thought": true,
                "with": true,
                "village": "include"
            }"#;

            let mut root_level = BTreeMap::new();

            let mut president = vec![];

            let mut president_inner_object = BTreeMap::new();
            president_inner_object.insert(
                "let".to_string(),
                JsonValue::Plain("1974033217".to_string()),
            );
            president_inner_object.insert(
                "nine".to_string(),
                JsonValue::Plain("1043848105".to_string()),
            );
            president_inner_object
                .insert("differ".to_string(), JsonValue::Plain("false".to_string()));
            president_inner_object.insert(
                "animal".to_string(),
                JsonValue::Plain("1083488261".to_string()),
            );
            president_inner_object
                .insert("section".to_string(), JsonValue::Plain("pale".to_string()));
            president_inner_object.insert("table".to_string(), JsonValue::Plain("fox".to_string()));
            president.push(JsonValue::Object(president_inner_object));

            president.push(JsonValue::Plain("false".to_string()));
            president.push(JsonValue::Plain("movie".to_string()));
            president.push(JsonValue::Plain("bee".to_string()));
            president.push(JsonValue::Plain("true".to_string()));
            president.push(JsonValue::Plain("-58333673.273878574".to_string()));

            root_level.insert("president".to_string(), JsonValue::Array(president));
            root_level.insert("thought".to_string(), JsonValue::Plain("true".to_string()));
            root_level.insert("with".to_string(), JsonValue::Plain("true".to_string()));
            root_level.insert(
                "village".to_string(),
                JsonValue::Plain("include".to_string()),
            );

            let expected_object = JsonValue::Object(root_level);

            let json = Json::new(&json_content);
            let actual_object = json.parse();

            assert_eq!(expected_object, actual_object);
        }
    }
}
