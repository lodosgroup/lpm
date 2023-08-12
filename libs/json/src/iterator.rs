use crate::json_value::JsonValue;

use std::collections::BTreeMap;

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
const TAB: char = '\t';

pub(crate) fn iterate_tokens(
    chars: &mut Vec<char>,
    root_object: &mut JsonValue,
    key_from_parent: Option<String>,
    mut parent_node: Option<&mut JsonValue>,
) -> Result<(), String> {
    let ptr: *const char = chars.as_ptr();

    // Controller value for when to parse key/value
    let key_passed_from_parent = key_from_parent.is_some();
    let mut is_key = false;
    let mut is_string_value = false;
    let mut is_non_string_value = false;

    let mut key = key_from_parent.unwrap_or_default();
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

        // Parse number and boolean values
        match current_char {
            OBJECT_OPENER if matches!(root_object, JsonValue::Null) => {
                *root_object = JsonValue::Object(BTreeMap::default());
            }

            ARRAY_OPENER if matches!(root_object, JsonValue::Null) => {
                *root_object = JsonValue::Array(vec![]);
            }

            // Ignore out-tree spaces/new lines
            NEWLINE | WHITESPACE | TAB if !is_key && !is_string_value => {
                chars.remove(pointer_cursor);
                chars_len -= 1;
                continue;
            }

            ARRAY_OPENER if !is_string_value => {
                chars.drain(0..pointer_cursor + 1);
                pointer_cursor = 0;

                let mut local_parent = if let Some(parent_node) = &mut parent_node {
                    if let JsonValue::Array(_) = parent_node {
                        parent_node.clone()
                    } else {
                        JsonValue::Array(vec![])
                    }
                } else {
                    JsonValue::Array(vec![])
                };

                iterate_tokens(
                    chars,
                    root_object,
                    Some(key.clone()),
                    Some(&mut local_parent),
                )?;

                if let Some(JsonValue::Object(parent_object)) = &mut parent_node {
                    parent_object.insert(key.clone(), local_parent);
                } else {
                    match root_object {
                        JsonValue::Plain(_) => {
                            return Err("Root object can not be `JsonValue::Plain`".to_string());
                        }
                        JsonValue::Object(object) => {
                            object.insert(key.clone(), local_parent);
                        }
                        JsonValue::Array(array) => {
                            array.push(local_parent);
                        }
                        JsonValue::Null => {
                            return Err("Root object can not be `JsonValue::Null`".to_string());
                        }
                    };
                };

                if !key_passed_from_parent {
                    is_key = false;
                    key = String::default();
                }

                is_non_string_value = false;
                is_string_value = false;
                value = String::default();

                chars_len = chars.len();
            }

            OBJECT_OPENER if !is_string_value => {
                chars.drain(0..pointer_cursor + 1);
                pointer_cursor = 0;

                let mut local_parent = JsonValue::Object(BTreeMap::new());
                iterate_tokens(chars, root_object, None, Some(&mut local_parent))?;

                match &mut parent_node {
                    Some(JsonValue::Array(parent_array)) => {
                        parent_array.push(local_parent);
                    }

                    Some(JsonValue::Object(parent_object)) => {
                        parent_object.insert(key.clone(), local_parent);
                    }
                    _ => {
                        match root_object {
                            JsonValue::Plain(_) => {
                                return Err("Root object can not be `JsonValue::Plain`".to_string());
                            }
                            JsonValue::Object(object) => {
                                object.insert(key.clone(), local_parent);
                            }
                            JsonValue::Array(array) => {
                                array.push(local_parent);
                            }
                            JsonValue::Null => {
                                return Err("Root object can not be `JsonValue::Null`".to_string());
                            }
                        };
                    }
                };

                if !key_passed_from_parent {
                    is_key = false;
                    key = String::default();
                }

                is_non_string_value = false;
                is_string_value = false;
                value = String::default();

                chars_len = chars.len();
            }

            OBJECT_CLOSER | ARRAY_CLOSER if !is_string_value => {
                // If last iteration built a value, then save it
                if is_non_string_value || is_string_value {
                    match &mut parent_node {
                        Some(JsonValue::Array(array)) => {
                            array.push(JsonValue::Plain(value.clone()));
                        }
                        Some(JsonValue::Object(object)) => {
                            object.insert(key, JsonValue::Plain(value.clone()));
                        }
                        Some(JsonValue::Plain(_)) => {
                            return Err("Parent node can not be `JsonValue::Plain`".to_string());
                        }
                        Some(JsonValue::Null) => {
                            return Err("Parent node can not be `JsonValue::Null`".to_string());
                        }
                        None => {
                            return Err("Parent node can not be empty".to_string());
                        }
                    };
                }

                // Consume used chars
                chars.drain(0..pointer_cursor);

                return Ok(());
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
                if let Some(JsonValue::Object(parent_object)) = &mut parent_node {
                    parent_object.insert(key.clone(), JsonValue::Plain(value));
                } else {
                    match root_object {
                        JsonValue::Object(object) => {
                            object.insert(key.clone(), JsonValue::Plain(value));
                        }
                        JsonValue::Array(array) => {
                            array.push(JsonValue::Plain(value));
                        }
                        JsonValue::Plain(_) => {
                            return Err("Root object can not be `JsonValue::Plain`".to_string());
                        }
                        JsonValue::Null => {
                            return Err("Root object can not be `JsonValue::Null`".to_string());
                        }
                    };
                };

                is_key = false;
                is_string_value = false;
                key = String::default();
                value = String::default();
            }

            non_string_value_char
                if key_passed_from_parent && !is_string_value && current_char != COMMA =>
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

            COMMA if is_non_string_value && key_passed_from_parent => {
                match &mut parent_node {
                    Some(JsonValue::Array(array)) => {
                        array.push(JsonValue::Plain(value.clone()));
                    }
                    Some(JsonValue::Object(object)) => {
                        object.insert(key.clone(), JsonValue::Plain(value.clone()));
                    }
                    Some(JsonValue::Plain(_)) => {
                        return Err("Parent node can not be `JsonValue::Plain`".to_string());
                    }
                    Some(JsonValue::Null) => {
                        return Err("Parent node can not be `JsonValue::Null`".to_string());
                    }
                    None => {
                        return Err("Parent node can not be empty.".to_string());
                    }
                };
                is_non_string_value = false;
                value = String::default();
            }

            // ** only for non-string values **
            // if current char is comma but it's not a value of string
            COMMA if is_non_string_value && !is_key && !is_string_value => {
                if let Some(JsonValue::Object(parent_object)) = &mut parent_node {
                    if &value == "null" {
                        parent_object.insert(key.clone(), JsonValue::Null);
                    } else {
                        parent_object.insert(key.clone(), JsonValue::Plain(value));
                    }
                } else {
                    match root_object {
                        JsonValue::Object(object) => {
                            if &value == "null" {
                                object.insert(key.clone(), JsonValue::Null);
                            } else {
                                object.insert(key.clone(), JsonValue::Plain(value));
                            }
                        }
                        JsonValue::Array(array) => {
                            if &value == "null" {
                                array.push(JsonValue::Null);
                            } else {
                                array.push(JsonValue::Plain(value));
                            }
                        }
                        JsonValue::Plain(_) => {
                            return Err("Root object can not be `JsonValue::Plain`".to_string());
                        }
                        JsonValue::Null => {
                            return Err("Root object can not be `JsonValue::Null`".to_string());
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
            c if is_key && !is_string_value && !is_non_string_value && !key_passed_from_parent => {
                key.push(c)
            }

            // ** only for object and array values **
            c if key_passed_from_parent => value.push(c),

            // ** only for non-string values **
            // If char is part of the non-string value
            c if !is_key && !is_string_value && is_non_string_value => {
                value.push(c);

                if let Some(cc) = next_char() {
                    if [OBJECT_CLOSER, ARRAY_CLOSER, WHITESPACE, NEWLINE].contains(&cc) {
                        if let Some(JsonValue::Object(parent_object)) = &mut parent_node {
                            if &value == "null" {
                                parent_object.insert(key.clone(), JsonValue::Null);
                            } else {
                                parent_object.insert(key.clone(), JsonValue::Plain(value));
                            }
                        } else {
                            match root_object {
                                JsonValue::Object(object) => {
                                    if &value == "null" {
                                        object.insert(key.clone(), JsonValue::Null);
                                    } else {
                                        object.insert(key.clone(), JsonValue::Plain(value));
                                    }
                                }
                                JsonValue::Array(array) => {
                                    if &value == "null" {
                                        array.push(JsonValue::Null);
                                    } else {
                                        array.push(JsonValue::Plain(value));
                                    }
                                }
                                JsonValue::Plain(_) => {
                                    return Err(
                                        "Root object can not be `JsonValue::Plain`".to_string()
                                    );
                                }
                                JsonValue::Null => {
                                    return Err(
                                        "Root object can not be `JsonValue::Null`".to_string()
                                    );
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

            // Ignore colons that are not part of anything
            c if c == COLON => {}

            c => {
                return Err(format!(
                    "Unexpected iterator behaviour. Char '{}' was not handled",
                    c
                ));
            }
        };

        pointer_cursor += 1;
    }

    Ok(())
}
