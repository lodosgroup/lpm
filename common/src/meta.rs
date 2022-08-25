use crate::version::VersionStruct;
use crate::ParserTasks;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct Files(pub Vec<FileStruct>);

#[derive(Debug, Clone, Deserialize)]
pub struct DependencyStruct {
    pub name: String,
    pub version: VersionStruct,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SuggestionStruct {
    pub name: String,
    pub version: Option<VersionStruct>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileStruct {
    pub path: String,
    pub checksum_algorithm: String,
    pub checksum: String,
}

macro_rules! impl_parser_tasks {
    ($($t:ty),+) => {
        $(impl ParserTasks for $t {
            fn deserialize(path: &str) -> Self {
                let data_as_str =
                    fs::read_to_string(path).unwrap_or_else(|_| {
                        super::log_and_panic!(format!("{} could not found.", path));
                    });

                serde_json::from_str(&data_as_str)
                    .unwrap_or_else(|_| {
                        super::log_and_panic!("Failed to parse package meta.");
                    })
            }
        })+
    }
}

impl_parser_tasks!(Meta, Files);
