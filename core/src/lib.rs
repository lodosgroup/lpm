use std::path::Path;

use common::{lpm_version::get_lpm_version, pkg::LodPkg};
use db::{pkg::LodPkgCoreDbOps, DB_PATH};
use min_sqlite3_sys::prelude::*;

const EXTRACTION_OUTPUT_PATH: &str = "/var/cache/lpm";

pub trait AdditionalCapabilities {
    fn from_db(pkg_name: &str) -> Self;
}

impl<'a> AdditionalCapabilities for LodPkg<'a> {
    fn from_db(pkg_name: &str) -> Self {
        let instance = LodPkg::default();
        let db = Database::open(Path::new(DB_PATH)).unwrap();
        let _x = instance.get_by_name(&db, pkg_name).unwrap();

        db.close();
        // read package from the database if exists
        Self {
            path: std::path::Path::new("pkg_name"),
            meta_dir: None,
            system: None,
            version: get_lpm_version(),
        }
    }
}

pub mod deletion;
pub mod extraction;
pub mod installation;
pub mod validation;
