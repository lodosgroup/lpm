use std::{
    fs::{create_dir_all, remove_dir_all, File},
    io::copy,
    path::Path,
    str::from_utf8,
};

use lpm_io::file::copy_recursively;
use parser::{
    meta::{Checksums, Meta},
    system::System,
    ParserTasks,
};
use xz2::read::XzDecoder;

use crate::{ExtractionTasks, EXTRACTION_OUTPUT_PATH};

pub struct LodPkg<'a> {
    pub path: &'a Path,
    pub meta_dir: Option<MetaDir>,
    pub system: Option<System>,
}

#[derive(Debug)]
pub struct MetaDir {
    pub path: String,
    pub meta: Meta,
    pub checksums: Checksums,
}

impl MetaDir {
    pub fn new(str_path: &str) -> Self {
        Self {
            path: String::from(str_path),
            meta: Meta::deserialize(&(str_path.to_owned() + "/meta.json")),
            checksums: Checksums::deserialize(&(str_path.to_owned() + "/checksums.json")),
        }
    }
}

impl<'a> LodPkg<'a> {
    pub fn new(str_path: &'a str) -> Self {
        Self {
            path: Path::new(str_path),
            meta_dir: None,
            system: None,
        }
    }
}

impl<'a> ExtractionTasks for LodPkg<'a> {
    fn start_extraction(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.half_extract()?;
        self.extract_meta_and_program()?;
        self.read_pkg_data();
        self.install_program()?;
        self.cleanup()?;

        Ok(())
    }

    fn half_extract(&self) -> Result<(), std::io::Error> {
        let input_file = File::open(self.path).expect("Package could not opened.");
        let mut archive = ar::Archive::new(input_file);

        while let Some(entry) = archive.next_entry() {
            let mut entry = entry.expect("Failed on parsing archive entry.");
            let filename = from_utf8(entry.header().identifier())
                .expect("Package has a file that has non-utf8 name.");
            let mut output_path = EXTRACTION_OUTPUT_PATH.to_string()
                + "/"
                + self.path.file_stem().unwrap().to_str().unwrap();

            create_dir_all(output_path.clone())?;

            output_path += "/";
            output_path += filename;

            let output_path = Path::new(&output_path).to_path_buf();
            let mut output_file = File::create(&output_path)?;
            copy(&mut entry, &mut output_file)
                .expect(&format!("Failed to copy {:?}.", output_path));
        }

        Ok(())
    }

    fn extract_meta_and_program(&self) -> Result<(), std::io::Error> {
        let pkg_dir = EXTRACTION_OUTPUT_PATH.to_string()
            + "/"
            + self.path.file_stem().unwrap().to_str().unwrap();

        let tar_file_path = pkg_dir.clone() + "/meta.tar.xz";
        let tar_file = File::open(tar_file_path)?;
        let mut archive = tar::Archive::new(XzDecoder::new(tar_file));
        archive.unpack(&pkg_dir)?;

        let tar_file_path = pkg_dir.clone() + "/program.tar.xz";
        let tar_file = File::open(tar_file_path)?;
        let mut archive = tar::Archive::new(XzDecoder::new(tar_file));
        archive.unpack(&pkg_dir)?;

        Ok(())
    }

    fn read_pkg_data(&mut self) {
        let pkg_dir = EXTRACTION_OUTPUT_PATH.to_string()
            + "/"
            + self.path.file_stem().unwrap().to_str().unwrap();

        let meta_dir = pkg_dir.clone() + "/meta";
        let system_json = pkg_dir + "/system.json";

        self.meta_dir = Some(MetaDir::new(&meta_dir));
        self.system = Some(System::deserialize(&system_json));
    }

    fn install_program(&self) -> Result<(), std::io::Error> {
        let src = EXTRACTION_OUTPUT_PATH.to_string()
            + "/"
            + self.path.file_stem().unwrap().to_str().unwrap()
            + "/program/";

        copy_recursively(&src, "/")?;

        Ok(())
    }

    fn cleanup(&self) -> Result<(), std::io::Error> {
        let pkg_dir = EXTRACTION_OUTPUT_PATH.to_string()
            + "/"
            + self.path.file_stem().unwrap().to_str().unwrap();

        remove_dir_all(pkg_dir)?;
        Ok(())
    }
}
