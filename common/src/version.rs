use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct VersionStruct {
    pub readable_format: String,
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub tag: Option<String>,
}
