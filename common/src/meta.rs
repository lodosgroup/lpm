use crate::version::VersionStruct;
use crate::{de_required_field, ParserTasks};
use json::{Deserialize, JsonValue};
use std::fs;

#[derive(Debug, Clone)]
pub struct Meta {
    pub name: String,
    pub description: String,
    pub maintainer: String,
    pub source_pkg: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub arch: String, // maybe use enums
    pub kind: String,
    pub installed_size: i64,
    pub tags: Vec<String>,
    pub version: VersionStruct,
    pub license: Option<String>,
    pub dependencies: Vec<DependencyStruct>,
    pub suggestions: Vec<SuggestionStruct>,
}

impl json::Deserialize for Meta {
    type Error = String;

    fn from_json_object(json: &json::JsonValue) -> Result<Self, Self::Error> {
        let mut tags = vec![];
        match &json["tags"] {
            json::JsonValue::Plain(_) => todo!(),
            json::JsonValue::Object(_) => todo!(),
            json::JsonValue::Array(array) => {
                for item in array {
                    tags.push(item.to_string().unwrap());
                }
            }
            json::JsonValue::Null => todo!(),
        };

        let version = VersionStruct::from_json_object(&json["version"])?;
        let dependencies = DependencyStruct::from_json_array(&json["dependencies"])?;
        let suggestions = SuggestionStruct::from_json_array(&json["suggestions"])?;

        Ok(Self {
            name: json["name"].to_string().unwrap(),
            description: json["description"].to_string().unwrap(),
            maintainer: json["maintainer"].to_string().unwrap(),
            source_pkg: json["source_pkg"].to_string(),
            repository: json["repository"].to_string(),
            homepage: json["homepage"].to_string(),
            arch: json["arch"].to_string().unwrap(),
            kind: json["kind"].to_string().unwrap(),
            installed_size: json["installed_size"].as_i64().unwrap(),
            tags,
            version,
            license: json["license"].to_string(),
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
            json::JsonValue::Plain(_) => todo!(),
            json::JsonValue::Object(_) => todo!(),
            json::JsonValue::Array(array) => {
                for item in array {
                    vec.push(FileStruct::from_json_object(item)?)
                }
            }
            json::JsonValue::Null => todo!(),
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
            super::log_and_panic!("{} could not found.", path);
        });

        let json = json::Json::new(&data_as_str)
            .parse()
            .unwrap_or_else(|error| {
                term::debug!("Error: {}", error);
                super::log_and_panic!(
                    "Package is either invalid or corrupted. Failed deserializing meta data."
                );
            });

        Self::from_json_object(&json).unwrap_or_else(|error| {
            super::log_and_panic!("INTERNAL: {}", error);
        })
    }
}

impl ParserTasks for Files {
    fn deserialize(path: &str) -> Self {
        let data_as_str = fs::read_to_string(path).unwrap_or_else(|_| {
            super::log_and_panic!("{} could not found.", path);
        });

        let json = json::Json::new(&data_as_str)
            .parse()
            .unwrap_or_else(|error| {
                term::debug!("Error: {}", error);
                super::log_and_panic!(
                    "Package is either invalid or corrupted. Failed deserializing meta data."
                );
            });

        Self::from_json_object(&json).unwrap_or_else(|error| {
            term::debug!("Error: {}", error);
            super::log_and_panic!("INTERNAL: {}", error);
        })
    }
}
