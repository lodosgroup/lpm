use std::{error, fs, io::Read};

use ehandle::package::{PackageError, PackageErrorKind};
use hash::{md5, sha256, sha512};
use parser::meta::Checksums;

use crate::{pkg::LodPkg, extraction::ExtractionTasks};

#[non_exhaustive]
enum ChecksumKind {
    Md5,
    Sha256,
    Sha512,
}

#[allow(dead_code)]
impl ChecksumKind {
    pub fn as_str(&self) -> &str {
        match self {
            ChecksumKind::Md5 => "md5",
            ChecksumKind::Sha256 => "sha256",
            ChecksumKind::Sha512 => "sha512",
        }
    }

    pub fn from_str(kind: &str) -> Result<ChecksumKind, PackageError> {
        match kind {
            "md5" => Ok(ChecksumKind::Md5),
            "sha256" => Ok(ChecksumKind::Sha256),
            "sha512" => Ok(ChecksumKind::Sha512),
            _ => Err(PackageError::new(
                PackageErrorKind::UnsupportedChecksumAlgorithm,
            )),
        }
    }
}

pub trait ValidationTasks {
    fn start_validations(&self) -> Result<(), Box<dyn error::Error>>;
}

impl<'a> ValidationTasks for LodPkg<'a> {
    fn start_validations(&self) -> Result<(), Box<dyn error::Error>> {
        if let Some(meta_dir) = &self.meta_dir {
            check_program_checksums(self.get_pkg_output_path(), &meta_dir.checksums)?
        }

        Ok(())
    }
}

#[inline(always)]
fn check_program_checksums(
    dir_path: String,
    checksums: &Checksums,
) -> Result<(), Box<dyn error::Error>> {
    if let Ok(kind) = ChecksumKind::from_str(checksums.kind.to_lowercase().as_str()) {
        for file in &checksums.files {
            // Read file as byte-array
            let mut f_reader = fs::File::open(dir_path.clone() + "/program/" + &file.path)?;
            let mut buffer = Vec::new();
            f_reader.read_to_end(&mut buffer).unwrap();

            // Generate hash with using same algorithm of pkg checksum
            let file_hash = match kind {
                // TODO
                // support more algorithms which are more secure
                ChecksumKind::Md5 => hash::digest_to_hex_string(&md5::digest(&buffer)),
                ChecksumKind::Sha256 => hash::digest_to_hex_string(&sha256::digest(&buffer)),
                ChecksumKind::Sha512 => hash::digest_to_hex_string(&sha512::digest(&buffer)),
            };

            if file_hash.ne(&file.checksum) {
                return Err(PackageError::new(PackageErrorKind::InvalidPackageFiles).into());
            }
        }
    } else {
        return Err(PackageError::new(PackageErrorKind::UnsupportedChecksumAlgorithm).into());
    }

    Ok(())
}
