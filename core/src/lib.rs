use common::{lpm_version::get_lpm_version, pkg::LodPkg};

const EXTRACTION_OUTPUT_PATH: &str = "/var/cache/lpm";

pub trait AdditionalCapabilities {
    fn from_db(pkg_name: &str) -> Self;
}

impl<'a> AdditionalCapabilities for LodPkg<'a> {
    fn from_db(_pkg_name: &str) -> Self {
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
