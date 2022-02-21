use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct VersionStruct {
    pub readable_format: String,
    pub major: u8,
    pub minior: u8,
    pub patch: u8,
    pub tag: String,
}
