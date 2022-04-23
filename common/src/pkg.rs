use crate::lpm_version::get_lpm_version;
use parser::{
    meta::{Files, Meta},
    system::System,
    version::VersionStruct,
    ParserTasks,
};
use std::path::Path;

pub struct LodPkg<'a> {
    pub path: &'a Path,
    pub meta_dir: Option<MetaDir>,
    pub system: Option<System>,
    pub version: VersionStruct,
}

#[derive(Debug)]
pub struct MetaDir {
    pub path: String,
    pub meta: Meta,
    pub files: Files,
}

impl MetaDir {
    pub fn new(str_path: &str) -> Self {
        Self {
            path: String::from(str_path),
            meta: Meta::deserialize(&(str_path.to_owned() + "/meta.json")),
            files: Files::deserialize(&(str_path.to_owned() + "/files.json")),
        }
    }
}

impl<'a> LodPkg<'a> {
    pub fn new(str_path: &'a str) -> Self {
        Self {
            path: Path::new(str_path),
            meta_dir: None,
            system: None,
            version: get_lpm_version(),
        }
    }
}