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

pub enum Stage1Script {
    PreInstall(PathBuf),
    PostInstall(PathBuf),
    PreDelete(PathBuf),
    PostDelete(PathBuf),
    PreDowngrade(PathBuf),
    PostDowngrade(PathBuf),
    PreUpgrade(PathBuf),
    PostUpgrade(PathBuf),
}

impl Stage1Script {
    pub fn get_inner(&self) -> &Path {
        match self {
            Stage1Script::PreInstall(path)
            | Stage1Script::PostInstall(path)
            | Stage1Script::PreDelete(path)
            | Stage1Script::PostDelete(path)
            | Stage1Script::PreDowngrade(path)
            | Stage1Script::PostDowngrade(path)
            | Stage1Script::PreUpgrade(path)
            | Stage1Script::PostUpgrade(path) => path,
        }
    }
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
