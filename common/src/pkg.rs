use super::ParserTasks;
use crate::{
    meta::{Files, Meta},
    system::System,
};
use std::path::PathBuf;

pub struct PkgDataFromFs {
    pub path: PathBuf,
    pub meta_dir: MetaDir,
    pub system: System,
}

pub struct PkgDataFromDb {
    pub pkg_id: i64,
    pub meta_dir: MetaDir,
}

pub struct MetaDir {
    pub path: String,
    pub meta: Meta,
    pub files: Files,
}

impl MetaDir {
    #[inline]
    pub fn new(str_path: &str) -> Self {
        Self {
            path: String::from(str_path),
            meta: Meta::deserialize(&(str_path.to_owned() + "/meta.json")),
            files: Files::deserialize(&(str_path.to_owned() + "/files.json")),
        }
    }
}
