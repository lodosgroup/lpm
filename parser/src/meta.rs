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
    pub repository: String, // maybe make it optional
    pub homepage: Option<String>,
    pub arch: String, // maybe use enums
    pub kind: String,
    pub installed_size: u128,
    pub tags: Vec<String>,
    pub version: VersionStruct,
    pub license: Option<String>,
    pub dependencies: Vec<DependencyStruct>,
    pub suggestions: Vec<SuggestionStruct>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Checksums {
    pub kind: String,
    pub files: Vec<ChecksumFileStruct>,
}

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
pub struct ChecksumFileStruct {
    pub path: String,
    pub checksum: String,
}

macro_rules! impl_parser_tasks {
    ($($t:ty),+) => {
        $(impl ParserTasks for $t {
            fn deserialize(path: &str) -> Self {
                let data_as_str =
                    fs::read_to_string(path).unwrap_or_else(|_| panic!("{} could not found.", path));

                serde_json::from_str(&data_as_str)
                    //.unwrap()
                    .unwrap_or_else(|_| panic!("Failed to parse package meta."))
            }
        })+
    }
}

impl_parser_tasks!(Meta, Checksums);
