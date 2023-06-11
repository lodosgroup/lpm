use crate::version::VersionStruct;
use crate::{de_required_field, ParserTasks};

use json::{Deserialize, JsonValue};
use std::fs;

#[derive(Debug, Clone)]
pub struct Meta {
    pub name: String,
    pub arch: String, // TODO: use enums
    pub installed_size: i64,
    pub version: VersionStruct,
    pub dependencies: Vec<DependencyStruct>,
    pub suggestions: Vec<SuggestionStruct>,
}

impl json::Deserialize for Meta {
    type Error = String;

    fn from_json_object(json: &json::JsonValue) -> Result<Self, Self::Error> {
        let version = VersionStruct::from_json_object(&json["version"])?;
        let dependencies = DependencyStruct::from_json_array(&json["dependencies"])?;
        let suggestions = SuggestionStruct::from_json_array(&json["suggestions"])?;

        Ok(Self {
            name: de_required_field!(json["name"].to_string(), "name"),
            arch: de_required_field!(json["arch"].to_string(), "arch"),
            installed_size: de_required_field!(json["installed_size"].as_i64(), "installed_size"),
            version,
            dependencies,
            suggestions,
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

#[derive(Debug, Clone)]
pub struct Files(pub Vec<FileStruct>);

impl json::Deserialize for Files {
    type Error = String;

    fn from_json_object(json: &json::JsonValue) -> Result<Self, Self::Error> {
        let mut vec: Vec<FileStruct> = vec![];
        match json {
            json::JsonValue::Array(array) => {
                for item in array {
                    vec.push(FileStruct::from_json_object(item)?)
                }
            }
            _ => return Err("Wrong input, expected an array".to_string()),
        }

        Ok(Self(vec))
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

#[derive(Debug, Clone)]
pub struct DependencyStruct {
    pub name: String,
    pub version: VersionStruct,
}

impl json::Deserialize for DependencyStruct {
    type Error = String;

    fn from_json_object(json: &json::JsonValue) -> Result<Self, Self::Error> {
        Ok(Self {
            name: de_required_field!(json["name"].to_string(), "name"),
            version: VersionStruct::from_json_object(&json["version"])?,
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

#[derive(Debug, Clone)]
pub struct SuggestionStruct {
    pub name: String,
    pub version: Option<VersionStruct>,
}

impl json::Deserialize for SuggestionStruct {
    type Error = String;

    fn from_json_object(json: &json::JsonValue) -> Result<Self, Self::Error> {
        let version = if !json["version"].is_null() {
            let version: VersionStruct = VersionStruct::from_json_object(&json["version"])?;
            Some(version)
        } else {
            None
        };

        Ok(Self {
            name: de_required_field!(json["name"].to_string(), "name"),
            version,
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

#[derive(Debug, Clone)]
pub struct FileStruct {
    pub path: String,
    pub checksum_algorithm: String,
    pub checksum: String,
}

impl json::Deserialize for FileStruct {
    type Error = String;

    fn from_json_object(json: &json::JsonValue) -> Result<Self, Self::Error> {
        Ok(Self {
            path: de_required_field!(json["path"].to_string(), "path"),
            checksum_algorithm: de_required_field!(
                json["checksum_algorithm"].to_string(),
                "checksum_algorithm"
            ),
            checksum: de_required_field!(json["checksum"].to_string(), "checksum"),
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

impl ParserTasks for Meta {
    fn deserialize(path: &str) -> Self {
        let data_as_str = fs::read_to_string(path).unwrap_or_else(|_| {
            panic!("{} could not found.", path);
        });

        let json = json::Json::new(&data_as_str)
            .parse()
            .unwrap_or_else(|_error| {
                logger::debug!("Error: {}", _error);
                panic!("Package is either invalid or corrupted. Failed deserializing meta data.");
            });

        Self::from_json_object(&json).unwrap_or_else(|error| {
            panic!("INTERNAL: {}", error);
        })
    }
}

impl ParserTasks for Files {
    fn deserialize(path: &str) -> Self {
        let data_as_str = fs::read_to_string(path).unwrap_or_else(|_| {
            panic!("{} could not found.", path);
        });

        let json = json::Json::new(&data_as_str)
            .parse()
            .unwrap_or_else(|_error| {
                logger::debug!("Error: {}", _error);
                panic!("Package is either invalid or corrupted. Failed deserializing meta data.");
            });

        Self::from_json_object(&json).unwrap_or_else(|error| {
            logger::debug!("Error: {}", error);
            panic!("INTERNAL: {}", error);
        })
    }
}
