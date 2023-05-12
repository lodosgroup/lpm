use super::ParserTasks;
use crate::{
    meta::{Files, Meta},
    system::System,
};
use std::path::{Path, PathBuf};

pub struct PkgDataFromFs {
    pub path: PathBuf,
    pub meta_dir: MetaDir,
    pub scripts: Vec<Stage1Script>,
    pub system: System,
}

pub struct PkgDataFromDb {
    pub pkg_id: i64,
    pub meta_dir: MetaDir,
}

pub struct MetaDir {
    pub path: PathBuf,
    pub meta: Meta,
    pub files: Files,
}

#[derive(PartialEq)]
pub enum ScriptPhase {
    PreInstall,
    PostInstall,
    PreDelete,
    PostDelete,
    PreDowngrade,
    PostDowngrade,
    PreUpgrade,
    PostUpgrade,
}

pub struct Stage1Script {
    pub contents: String,
    pub path: PathBuf,
    pub phase: ScriptPhase,
}

impl MetaDir {
    pub fn new(dir: &Path) -> Self {
        Self {
            path: dir.to_owned(),
            meta: Meta::deserialize(&dir.join("meta.json").to_string_lossy()),
            files: Files::deserialize(&dir.join("files.json").to_string_lossy()),
        }
    }
}
