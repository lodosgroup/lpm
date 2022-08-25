use common::{
    pkg::{LodPkg, MetaDir},
    system::System,
    ParserTasks,
};
use ehandle::lpm::LpmError;
use std::{
    fs::{create_dir_all, remove_dir_all, File},
    io::{self, copy},
    path::Path,
    str::from_utf8,
};
use term::debug;
use xz2::read::XzDecoder;

pub trait ExtractionTasks {
    fn start_extraction(&mut self) -> Result<(), LpmError<io::Error>>;
    fn get_pkg_output_path(&self) -> String;
    fn half_extract(&self) -> Result<(), LpmError<io::Error>>;
    fn extract_meta_and_program(&self) -> Result<(), LpmError<io::Error>>;
    fn read_pkg_data(&mut self);
    fn cleanup(&self) -> Result<(), LpmError<io::Error>>;
}

impl<'a> ExtractionTasks for LodPkg<'a> {
    fn start_extraction(&mut self) -> Result<(), LpmError<io::Error>> {
        self.half_extract()?;
        self.extract_meta_and_program()?;
        self.read_pkg_data();

        Ok(())
    }

    #[inline]
    fn get_pkg_output_path(&self) -> String {
        super::EXTRACTION_OUTPUT_PATH.to_string()
            + "/"
            + self.path.unwrap().file_stem().unwrap().to_str().unwrap()
    }

    fn half_extract(&self) -> Result<(), LpmError<io::Error>> {
        let input_file = File::open(self.path.unwrap())?;
        let mut archive = ar::Archive::new(input_file);

        while let Some(entry) = archive.next_entry() {
            let mut entry = entry?;
            let filename = from_utf8(entry.header().identifier()).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Package has a file with non UTF-8 filename.",
                )
            })?;
            let mut output_path = self.get_pkg_output_path();

            create_dir_all(output_path.clone())?;

            output_path += "/";
            output_path += filename;

            debug!("Extracting {} -> {}", filename, output_path);

            let output_path = Path::new(&output_path).to_path_buf();
            let mut output_file = File::create(&output_path)?;

            copy(&mut entry, &mut output_file)?;
        }

        Ok(())
    }

    fn extract_meta_and_program(&self) -> Result<(), LpmError<io::Error>> {
        let pkg_dir = self.get_pkg_output_path();

        let tar_file_path = pkg_dir.clone() + "/meta.tar.xz";
        let tar_file = File::open(&tar_file_path)?;
        debug!("Extracting {} -> {}", tar_file_path, pkg_dir);
        let mut archive = tar::Archive::new(XzDecoder::new(tar_file));
        archive.unpack(&pkg_dir)?;

        let tar_file_path = pkg_dir.clone() + "/program.tar.xz";
        let tar_file = File::open(&tar_file_path)?;
        debug!("Extracting {} -> {}", tar_file_path, pkg_dir);
        let mut archive = tar::Archive::new(XzDecoder::new(tar_file));
        archive.unpack(&pkg_dir)?;

        Ok(())
    }

    fn read_pkg_data(&mut self) {
        let pkg_dir = self.get_pkg_output_path();

        let meta_dir = pkg_dir.clone() + "/meta";
        let system_json = pkg_dir + "/system.json";

        debug!(
            "Reading meta data from {}/meta.json and {}/files.json",
            &meta_dir, &meta_dir
        );
        self.meta_dir = Some(MetaDir::new(&meta_dir));
        debug!("Reading system data from {}", &system_json);
        self.system = Some(System::deserialize(&system_json));
    }

    fn cleanup(&self) -> Result<(), LpmError<io::Error>> {
        let pkg_dir = self.get_pkg_output_path();

        remove_dir_all(pkg_dir)?;
        Ok(())
    }
}
