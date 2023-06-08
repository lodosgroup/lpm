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
    pub meta_fields: MetaDir,
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

#[derive(Debug, PartialEq)]
pub struct PkgToQuery {
    pub name: String,
    pub major: Option<u32>,
    pub minor: Option<u32>,
    pub patch: Option<u32>,
    pub tag: Option<String>,
}

impl PkgToQuery {
    pub fn parse(pkg_name: &str) -> Option<Self> {
        let parts: Vec<&str> = pkg_name.split('@').collect();

        if parts.len() > 2 {
            return None;
        }

        let name = parts[0].to_string();
        let version = parts.get(1).copied();

        if let Some(version) = version {
            let mut version_parts = version.split('-');
            let version_numbers: Vec<&str> = version_parts.next()?.split('.').collect();

            let major = version_numbers[0].parse::<u32>().ok();
            let minor = version_numbers.get(1).and_then(|v| v.parse::<u32>().ok());
            let patch = version_numbers.get(2).and_then(|v| v.parse::<u32>().ok());
            let tag = version_parts.next().map(|v| v.to_string());

            Some(Self {
                name,
                major,
                minor,
                patch,
                tag,
            })
        } else {
            Some(Self {
                name,
                major: None,
                minor: None,
                patch: None,
                tag: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkg_to_query_with_version() {
        let pkg_name = "htop@1.3.5-beta";
        let package = PkgToQuery::parse(pkg_name).unwrap();

        assert_eq!(package.name, "htop");
        assert_eq!(package.major, Some(1));
        assert_eq!(package.minor, Some(3));
        assert_eq!(package.patch, Some(5));
        assert_eq!(package.tag, Some("beta".to_string()));
    }

    #[test]
    fn test_pkg_to_query_without_version() {
        let pkg_name = "htop";
        let package = PkgToQuery::parse(pkg_name).unwrap();

        assert_eq!(package.name, "htop");
        assert_eq!(package.major, None);
        assert_eq!(package.minor, None);
        assert_eq!(package.patch, None);
        assert_eq!(package.tag, None);
    }

    #[test]
    fn test_pkg_to_query_invalid_format() {
        let pkg_name = "htop@1.3.5-beta@invalid";
        let package = PkgToQuery::parse(pkg_name);

        assert_eq!(package, None);
    }

    #[test]
    fn test_pkg_to_query_with_major_version_only() {
        let pkg_name = "htop@1";
        let package = PkgToQuery::parse(pkg_name).unwrap();

        assert_eq!(package.name, "htop");
        assert_eq!(package.major, Some(1));
        assert_eq!(package.minor, None);
        assert_eq!(package.patch, None);
        assert_eq!(package.tag, None);
    }
}
