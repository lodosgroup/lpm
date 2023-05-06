use super::ParserTasks;
use crate::{log_and_panic, version::VersionStruct};
use json::{Deserialize, JsonValue};
use std::fs;

#[derive(Debug, Clone)]
pub struct System {
    pub builder_version: VersionStruct,
    pub min_supported_lpm_version: VersionStruct,
}

impl json::Deserialize for System {
    type Error = String;

    fn from_json_object(json: &json::JsonValue) -> Result<Self, Self::Error> {
        Ok(Self {
            builder_version: VersionStruct::from_json_object(&json["builder_version"])?,
            min_supported_lpm_version: VersionStruct::from_json_object(
                &json["min_supported_lpm_version"],
            )?,
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

impl ParserTasks for System {
    fn deserialize(path: &str) -> Self {
        let data_as_str = fs::read_to_string(path).unwrap_or_else(|_| {
            log_and_panic!("{} could not found.", path);
        });

        let json = json::Json::new(&data_as_str)
            .parse()
            .unwrap_or_else(|_error| {
                logger::debug!("Error: {}", _error);
                super::log_and_panic!(
                    "Package is either invalid or corrupted. Failed deserializing system data."
                );
            });

        Self::from_json_object(&json).unwrap_or_else(|error| {
            super::log_and_panic!("INTERNAL: {}", error);
        })
    }
}
