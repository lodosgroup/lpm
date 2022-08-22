use crate::extraction::ExtractionTasks;
use common::meta::Files;
use common::{pkg::LodPkg, NO_ARCH, SYSTEM_ARCH};
use ehandle::lpm::LpmError;
use ehandle::{
    pkg::{PackageError, PackageErrorKind},
    simple_e_fmt, ErrorCommons, RuntimeError,
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
            _ => Err(PackageErrorKind::UnsupportedChecksumAlgorithm(Some(format!(
                "{} algorithm is not supported from current lpm version.",
                kind
            )))
            .throw()),
        }
    }
}

pub trait ValidationTasks {
    fn start_validations(&self) -> Result<(), LpmError<RuntimeError>>;
}

impl<'a> ValidationTasks for LodPkg<'a> {
    fn start_validations(&self) -> Result<(), LpmError<RuntimeError>> {
        if let Some(meta_dir) = &self.meta_dir {
            // check architecture compatibility
            if meta_dir.meta.arch != NO_ARCH && meta_dir.meta.arch != SYSTEM_ARCH {
                let e = format!("Package '{}' is built for '{}' architecture that is not supported by this machine.", meta_dir.meta.name, meta_dir.meta.arch);
                return Err(LpmError::new(
                    PackageErrorKind::UnsupportedPackageArchitecture(Some(e)).throw(),
                )
                .into());
            }

            check_program_checksums(self.get_pkg_output_path(), &meta_dir.files)?
        }

        Ok(())
    }
}

fn check_program_checksums(dir_path: String, files: &Files) -> Result<(), LpmError<RuntimeError>> {
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
                return Err(LpmError::new(
                    PackageErrorKind::InvalidPackageFiles(Some(simple_e_fmt!(
                        "File \"{}\" is not valid.",
                        file.path
                    )))
                    .throw(),
                )
                .into());
            }
        } else {
            return Err(LpmError::new(
                PackageErrorKind::UnsupportedChecksumAlgorithm(Some(simple_e_fmt!(
                    "Algorithm \"{}\" is not supported by lpm.",
                    file.checksum_algorithm
                )))
                .throw(),
            )
            .into());
        }
    }

    Ok(())
}
