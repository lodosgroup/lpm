use serde::Deserialize;
use std::fs;

use super::ParserTasks;
use crate::version::VersionStruct;

#[derive(Debug, Clone, Deserialize)]
pub struct System {
    pub lod_version: VersionStruct,
}

impl ParserTasks for System {
    fn deserialize(path: &str) -> Self {
        let data_as_str =
            fs::read_to_string(path).unwrap_or_else(|_| panic!("{} could not found.", path));

        serde_json::from_str(&data_as_str)
            .unwrap_or_else(|_| panic!("Failed to parse package system."))
    }
}
