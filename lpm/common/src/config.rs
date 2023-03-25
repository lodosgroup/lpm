use crate::{de_required_field, ParserTasks};
use json::{Deserialize, JsonValue};
use std::fs;

#[cfg(not(debug_assertions))]
pub const CONFIG_PATH: &str = "/etc/lpm/conf";

#[cfg(debug_assertions)]
pub const CONFIG_PATH: &str = "conf";

pub struct LpmConfig {
    pub plugins: Vec<Plugin>,
}

pub struct Plugin {
    pub name: String,
    pub dylib_path: String,
}

impl json::Deserialize for Plugin {
    type Error = String;

    fn from_json_object(json: &json::JsonValue) -> Result<Self, Self::Error> {
        Ok(Self {
            name: de_required_field!(json["name"].to_string(), "name"),
            dylib_path: de_required_field!(json["dylib_path"].to_string(), "dylib_path"),
        })
    }

    fn from_json_array(json: &json::JsonValue) -> Result<Vec<Self>, Self::Error> {
        let mut object_array = vec![];
        match json {
            JsonValue::Array(array) => {
                for item in array {
                    let object = Self::from_json_object(item)?;
                    object_array.push(object);
                }
            }
            _ => return Err("Wrong input, expected an array".to_string()),
        };

        Ok(object_array)
    }
}

impl json::Deserialize for LpmConfig {
    type Error = String;

    fn from_json_object(json: &json::JsonValue) -> Result<Self, Self::Error> {
        let plugins = Plugin::from_json_array(&json["plugins"])?;

        Ok(Self { plugins })
    }

    fn from_json_array(json: &json::JsonValue) -> Result<Vec<Self>, Self::Error> {
        let mut object_array = vec![];
        match json {
            JsonValue::Array(array) => {
                for item in array {
                    let object = Self::from_json_object(item)?;
                    object_array.push(object);
                }
            }
            _ => return Err("Wrong input, expected an array".to_string()),
        };

        Ok(object_array)
    }
}

impl ParserTasks for LpmConfig {
    fn deserialize(path: &str) -> Self {
        let data_as_str = fs::read_to_string(path).unwrap_or_else(|_| {
            super::log_and_panic!("{} could not found.", path);
        });

        let json = json::Json::new(&data_as_str)
            .parse()
            .unwrap_or_else(|_error| {
                term::debug!("Error: {}", _error);
                super::log_and_panic!(
                    "Package is either invalid or corrupted. Failed deserializing lpm config."
                );
            });

        Self::from_json_object(&json).unwrap_or_else(|error| {
            super::log_and_panic!("INTERNAL: {}", error);
        })
    }
}
