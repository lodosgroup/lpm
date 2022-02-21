use serde::Deserialize;
use crate::version::VersionStruct;

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
