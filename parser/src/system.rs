use serde::Deserialize;
use crate::version::VersionStruct;

#[derive(Debug, Clone, Deserialize)]
pub struct System {
    pub lod_version: VersionStruct,
}
