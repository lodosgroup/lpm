use super::ParserTasks;
use crate::{
    meta::{Files, Meta},
    system::System,
    version::Condition,
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
    pub condition: Condition,
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
            let mut condition = Condition::default();
            let mut version_numbers: Vec<&str> = Vec::new();

            if let Some(part) = version_parts.next() {
                if part.starts_with(">=") || part.starts_with("<=") {
                    condition = Condition::from_string_slice(&part[..2]);
                    version_numbers = part[2..].split('.').collect();
                } else if part.starts_with('>') || part.starts_with('<') {
                    condition = Condition::from_string_slice(&part[..1]);
                    version_numbers = part[1..].split('.').collect();
                } else if let Some(stripped) = part.strip_prefix('=') {
                    condition = Condition::from_string_slice(&part[0..1]);
                    version_numbers = stripped.split('.').collect();
                } else {
                    version_numbers = part.split('.').collect();
                }
            }

            let major = version_numbers[0].parse::<u32>().ok();
            let minor = version_numbers.get(1).and_then(|v| v.parse::<u32>().ok());
            let patch = version_numbers.get(2).and_then(|v| v.parse::<u32>().ok());
            let tag = version_parts.next().map(|v| v.to_string());

            Some(Self {
                name,
                condition,
                major,
                minor,
                patch,
                tag,
            })
        } else {
            Some(Self {
                name,
                condition: Condition::default(),
                major: None,
                minor: None,
                patch: None,
                tag: None,
            })
        }
    }
}

impl ToString for PkgToQuery {
    fn to_string(&self) -> String {
        let mut s = self.name.clone();
        if let Some(v) = self.major {
            s = format!("{s}@{v}");
        }

        if let Some(v) = self.minor {
            s = format!("{s}.{v}");
        }

        if let Some(v) = self.patch {
            s = format!("{s}.{v}");
        }

        if let Some(v) = &self.tag {
            s = format!("{s}-{v}");
        }

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkg_to_query_with_version() {
        let pkg_name = "htop@1.3.5-beta";
        let actual = PkgToQuery::parse(pkg_name).unwrap();

        let expected = PkgToQuery {
            name: String::from("htop"),
            major: Some(1),
            minor: Some(3),
            patch: Some(5),
            tag: Some(String::from("beta")),
            condition: Condition::default(),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_pkg_to_query_with_condition_operator() {
        {
            let pkg_name = "htop@=1.3.5-beta";
            let actual = PkgToQuery::parse(pkg_name).unwrap();

            let expected = PkgToQuery {
                name: String::from("htop"),
                major: Some(1),
                minor: Some(3),
                patch: Some(5),
                tag: Some(String::from("beta")),
                condition: Condition::Equal,
            };

            assert_eq!(actual, expected);
        }

        {
            let pkg_name = "htop@<1.3.5-beta";
            let actual = PkgToQuery::parse(pkg_name).unwrap();

            let expected = PkgToQuery {
                name: String::from("htop"),
                major: Some(1),
                minor: Some(3),
                patch: Some(5),
                tag: Some(String::from("beta")),
                condition: Condition::Less,
            };

            assert_eq!(actual, expected);
        }

        {
            let pkg_name = "htop@>1.3.5-beta";
            let actual = PkgToQuery::parse(pkg_name).unwrap();

            let expected = PkgToQuery {
                name: String::from("htop"),
                major: Some(1),
                minor: Some(3),
                patch: Some(5),
                tag: Some(String::from("beta")),
                condition: Condition::Greater,
            };

            assert_eq!(actual, expected);
        }

        {
            let pkg_name = "htop@<=1.3.5-beta";
            let actual = PkgToQuery::parse(pkg_name).unwrap();

            let expected = PkgToQuery {
                name: String::from("htop"),
                major: Some(1),
                minor: Some(3),
                patch: Some(5),
                tag: Some(String::from("beta")),
                condition: Condition::LessOrEqual,
            };

            assert_eq!(actual, expected);
        }

        {
            let pkg_name = "htop@>=1.3.5-beta";
            let actual = PkgToQuery::parse(pkg_name).unwrap();

            let expected = PkgToQuery {
                name: String::from("htop"),
                major: Some(1),
                minor: Some(3),
                patch: Some(5),
                tag: Some(String::from("beta")),
                condition: Condition::GreaterOrEqual,
            };

            assert_eq!(actual, expected);
        }
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
