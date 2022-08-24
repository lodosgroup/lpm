use crate::extraction::ExtractionTasks;
use common::meta::Files;
use common::{pkg::LodPkg, NO_ARCH, SYSTEM_ARCH};
use ehandle::lpm::LpmError;
use ehandle::{
    pkg::{PackageError, PackageErrorKind},
    ErrorCommons, MainError,
};
use hash::{md5, sha256, sha512};
use std::{fs, io::Read};

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
            _ => Err(PackageErrorKind::UnsupportedChecksumAlgorithm(kind.to_string()).to_err()),
        }
    }
}

pub trait ValidationTasks {
    fn start_validations(&self) -> Result<(), LpmError<MainError>>;
}

impl<'a> ValidationTasks for LodPkg<'a> {
    fn start_validations(&self) -> Result<(), LpmError<MainError>> {
        if let Some(meta_dir) = &self.meta_dir {
            // check architecture compatibility
            if meta_dir.meta.arch != NO_ARCH && meta_dir.meta.arch != SYSTEM_ARCH {
                return Err(PackageErrorKind::UnsupportedPackageArchitecture(
                    meta_dir.meta.arch.clone(),
                )
                .to_lpm_err()
                .into());
            }

            check_program_checksums(self.get_pkg_output_path(), &meta_dir.files)?
        }

        Ok(())
    }
}

fn check_program_checksums(dir_path: String, files: &Files) -> Result<(), LpmError<MainError>> {
    for file in &files.0 {
        // Read file as byte-array
        let mut f_reader = fs::File::open(dir_path.clone() + "/program/" + &file.path)?;
        let mut buffer = Vec::new();
        f_reader.read_to_end(&mut buffer).unwrap();

        if let Ok(checksum_algorithm) =
            ChecksumKind::from_str(file.checksum_algorithm.to_lowercase().as_str())
        {
            // Generate hash with using same algorithm of pkg checksum
            let file_hash = match checksum_algorithm {
                ChecksumKind::Md5 => hash::digest_to_hex_string(&md5::digest(&buffer)),
                ChecksumKind::Sha256 => hash::digest_to_hex_string(&sha256::digest(&buffer)),
                ChecksumKind::Sha512 => hash::digest_to_hex_string(&sha512::digest(&buffer)),
            };

            if file_hash.ne(&file.checksum) {
                return Err(PackageErrorKind::InvalidPackageFiles.to_lpm_err().into());
            }
        } else {
            return Err(PackageErrorKind::UnsupportedChecksumAlgorithm(
                file.checksum_algorithm.clone(),
            )
            .to_lpm_err()
            .into());
        }
    }

    Ok(())
}
