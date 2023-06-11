use crate::extract::get_pkg_tmp_output_path;

use common::meta::Files;
use common::pkg::PkgDataFromFs;
use common::{NO_ARCH, SYSTEM_ARCH};
use ehandle::lpm::LpmError;
use ehandle::{
    pkg::{PackageError, PackageErrorKind},
    ErrorCommons, MainError,
};
use hash::{md5, sha256, sha512};
use logger::debug;
use std::fmt;
use std::path::Path;
use std::{fs, io::Read};

#[non_exhaustive]
enum ChecksumKind {
    Md5,
    Sha256,
    Sha512,
}

impl fmt::Display for ChecksumKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChecksumKind::Md5 => write!(f, "md5"),
            ChecksumKind::Sha256 => write!(f, "sha256"),
            ChecksumKind::Sha512 => write!(f, "sha512"),
        }
    }
}

impl ChecksumKind {
    pub fn from_str(kind: &str) -> Result<ChecksumKind, PackageError> {
        match kind {
            "md5" => Ok(ChecksumKind::Md5),
            "sha256" => Ok(ChecksumKind::Sha256),
            "sha512" => Ok(ChecksumKind::Sha512),
            _ => Err(PackageErrorKind::UnsupportedChecksumAlgorithm(kind.to_string()).to_err()),
        }
    }
}

pub(crate) trait PkgValidateTasks {
    fn start_validate_task(&self) -> Result<(), LpmError<MainError>>;
}

impl PkgValidateTasks for PkgDataFromFs {
    fn start_validate_task(&self) -> Result<(), LpmError<MainError>> {
        if self.meta_dir.meta.arch != NO_ARCH && self.meta_dir.meta.arch != SYSTEM_ARCH {
            return Err(PackageErrorKind::UnsupportedPackageArchitecture(
                self.meta_dir.meta.arch.clone(),
            )
            .to_lpm_err())?;
        }

        let pkg_output_path = get_pkg_tmp_output_path(&self.path);
        check_program_checksums(&pkg_output_path, &self.meta_dir.files)
    }
}

fn check_program_checksums(dir: &Path, files: &Files) -> Result<(), LpmError<MainError>> {
    for file in &files.0 {
        // Read file as byte-array
        let f_path = dir.join("program").join(&file.path);
        debug!("Reading {} in byte format", &f_path.display());
        let mut f_reader = fs::File::open(&f_path)?;
        let mut buffer = Vec::new();
        f_reader.read_to_end(&mut buffer)?;

        if let Ok(checksum_algorithm) =
            ChecksumKind::from_str(file.checksum_algorithm.to_lowercase().as_str())
        {
            debug!(
                "Checksum algorithm of {} is specified as {}",
                &f_path.display(),
                checksum_algorithm
            );
            // Generate hash with using same algorithm of pkg checksum
            let file_hash = match checksum_algorithm {
                ChecksumKind::Md5 => hash::digest_to_hex_string(&md5::digest(&buffer)),
                ChecksumKind::Sha256 => hash::digest_to_hex_string(&sha256::digest(&buffer)),
                ChecksumKind::Sha512 => hash::digest_to_hex_string(&sha512::digest(&buffer)),
            };

            debug!(
                "Checking checksum value of {} if it's corrupted or not",
                &f_path.display()
            );
            if file_hash.ne(&file.checksum) {
                return Err(PackageErrorKind::InvalidPackageFiles.to_lpm_err())?;
            }
        } else {
            return Err(PackageErrorKind::UnsupportedChecksumAlgorithm(
                file.checksum_algorithm.clone(),
            )
            .to_lpm_err())?;
        }
    }

    Ok(())
}
