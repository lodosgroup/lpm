use super::ParserTasks;
use crate::{
    meta::{Files, Meta},
    system::System,
};
use std::path::{Path, PathBuf};

pub struct PkgDataFromFs {
    pub path: PathBuf,
    pub meta_dir: MetaDir,
    pub scripts_dir: ScriptsDir,
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

/// Contains stage1 scripts for package operations. Each script
/// defined inside of `Option` type to return `None` if package doesn't
/// contain that script.
pub struct ScriptsDir {
    pub pre_install: Option<PathBuf>,
    pub post_install: Option<PathBuf>,
    pub pre_delete: Option<PathBuf>,
    pub post_delete: Option<PathBuf>,
    pub pre_downgrade: Option<PathBuf>,
    pub post_downgrade: Option<PathBuf>,
    pub pre_upgrade: Option<PathBuf>,
    pub post_upgrade: Option<PathBuf>,
}

impl ScriptsDir {
    pub fn new(extracted_pkg_path: &Path) -> Self {
        let pre_install = extracted_pkg_path.join("scripts").join("pre_install");
        let post_install = extracted_pkg_path.join("scripts").join("post_install");
        let pre_delete = extracted_pkg_path.join("scripts").join("pre_delete");
        let post_delete = extracted_pkg_path.join("scripts").join("post_delete");
        let pre_downgrade = extracted_pkg_path.join("scripts").join("pre_downgrade");
        let post_downgrade = extracted_pkg_path.join("scripts").join("post_downgrade");
        let pre_upgrade = extracted_pkg_path.join("scripts").join("pre_upgrade");
        let post_upgrade = extracted_pkg_path.join("scripts").join("post_upgrade");

        Self {
            pre_install: if pre_install.exists() {
                Some(pre_install)
            } else {
                None
            },
            post_install: if post_install.exists() {
                Some(post_install)
            } else {
                None
            },
            pre_delete: if pre_delete.exists() {
                Some(pre_delete)
            } else {
                None
            },
            post_delete: if post_delete.exists() {
                Some(post_delete)
            } else {
                None
            },
            pre_downgrade: if pre_downgrade.exists() {
                Some(pre_downgrade)
            } else {
                None
            },
            post_downgrade: if post_downgrade.exists() {
                Some(post_downgrade)
            } else {
                None
            },
            pre_upgrade: if pre_upgrade.exists() {
                Some(pre_upgrade)
            } else {
                None
            },
            post_upgrade: if post_upgrade.exists() {
                Some(post_upgrade)
            } else {
                None
            },
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
