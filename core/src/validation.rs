use parser::meta::Checksums;

use crate::pkg::LodPkg;

enum ChecksumKind {
    Md5,
    Sha256,
    Sha512,
}

impl ChecksumKind {
    pub fn as_str(&self) -> &str {
        match self {
            ChecksumKind::Md5 => "md5",
            ChecksumKind::Sha256 => "sha256",
            ChecksumKind::Sha512 => "sha512",
        }
    }

    // Provide error in Result
    pub fn from_str(kind: &str) -> Result<ChecksumKind, ()> {
        match kind {
            "md5" => Ok(ChecksumKind::Md5),
            "sha256" => Ok(ChecksumKind::Sha256),
            "sha512" => Ok(ChecksumKind::Sha512),
            _ => todo!(),
        }
    }
}

impl<'a> super::ValidationTasks for LodPkg<'a> {
    fn checksum_validation(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.meta_dir {
            Some(meta_dir) => validate_file_checksums(&meta_dir.checksums),
            // TODO
            None => {}
        }

        Ok(())
    }
}

#[inline(always)]
fn validate_file_checksums(checksums: &Checksums) {
    match ChecksumKind::from_str(checksums.kind.to_lowercase().as_str()) {
        // TODO
        Ok(_) => todo!(),
        // TODO
        Err(_) => todo!(),
    }
}
