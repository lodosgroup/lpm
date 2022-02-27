use std::{
    fs::{create_dir_all, File},
    io::copy,
    path::Path,
    str::from_utf8,
};

use parser::{
    meta::{Checksums, Meta},
    system::System,
    ParserTasks,
};

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
            meta: Meta::deserialize(str_path),
            checksums: Checksums::deserialize(str_path),
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
    fn half_extract(&self) -> Result<(), std::io::Error> {
        let input_file = File::open(self.path).expect("Package could not opened.");
        let mut archive = ar::Archive::new(input_file);

        while let Some(entry) = archive.next_entry() {
            let mut entry = entry.expect("Failed on parsing archive entry.");
            let filename = from_utf8(entry.header().identifier())
                .expect("Package has a file that has non-utf8 name.");
            let mut output_path = EXTRACTION_OUTPUT_PATH.to_string()
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

    fn read_pkg_data(&mut self) {
        self.meta_dir = Some(MetaDir::new(self.path.to_str().unwrap()));
        self.system = Some(System::deserialize(self.path.to_str().unwrap()));
    }
}
