use std::path::Path;

use common::pkg::LodPkg;
use db::{pkg::LodPkgCoreDbOps, DB_PATH};
use min_sqlite3_sys::prelude::*;

const EXTRACTION_OUTPUT_PATH: &str = "/var/cache/lpm";

pub trait AdditionalCapabilities {
    fn from_db(pkg_name: &str) -> Self;
}

impl<'a> AdditionalCapabilities for LodPkg<'a> {
    fn from_db(pkg_name: &str) -> Self {
        let db = Database::open(Path::new(DB_PATH)).unwrap();
        let pkg = LodPkg::get_by_name(&db, pkg_name).unwrap();
        db.close();

        pkg
    }
}

pub mod deletion;
pub mod extraction;
pub mod installation;
pub mod validation;
