use common::{
    pkg::{MetaDir, PkgDataFromFs},
    system::System,
    ParserTasks,
};
use ehandle::lpm::LpmError;
use std::{
    fs::{remove_dir_all, File},
    io,
    path::Path,
};
use term::debug;

pub trait PkgExtractTasks {
    fn start_extract_task(pkg_path: &Path) -> Result<Self, LpmError<io::Error>>
    where
        Self: Sized;
    fn unpack_and_decompress(pkg_path: &Path) -> Result<(), LpmError<io::Error>>;
    fn read_pkg_data(pkg_path: &Path) -> PkgDataFromFs;
    fn cleanup(&self) -> Result<(), LpmError<io::Error>>;
}

impl PkgExtractTasks for PkgDataFromFs {
    fn start_extract_task(pkg_path: &Path) -> Result<Self, LpmError<io::Error>>
    where
        Self: Sized,
    {
        PkgDataFromFs::unpack_and_decompress(pkg_path)?;
        let pkg_data = PkgDataFromFs::read_pkg_data(pkg_path);

        Ok(pkg_data)
    }

    fn unpack_and_decompress(pkg_path: &Path) -> Result<(), LpmError<io::Error>> {
        let compressed_pkg_file = File::open(pkg_path)?;
        let mut archive =
            untar::Archive::new(tiny_lz4_decoder_sys::Decoder::new(compressed_pkg_file)?);
        let tmp_dir = get_pkg_output_path(pkg_path);

        debug!("Extracting {} -> {}", pkg_path.display(), tmp_dir);
        archive.unpack(&tmp_dir)?;

        Ok(())
    }

    fn read_pkg_data(pkg_path: &Path) -> PkgDataFromFs {
        let pkg_dir = get_pkg_output_path(pkg_path);

        let meta_dir = pkg_dir.clone() + "/meta";
        let system_json = pkg_dir + "/system.json";

        debug!(
            "Reading meta data from {}/meta.json and {}/files.json",
            &meta_dir, &meta_dir
        );
        let meta_dir = MetaDir::new(&meta_dir);
        debug!("Reading system data from {}", &system_json);
        let system = System::deserialize(&system_json);
        PkgDataFromFs {
            path: pkg_path.to_path_buf(),
            meta_dir,
            system,
        }
    }

    fn cleanup(&self) -> Result<(), LpmError<io::Error>> {
        let path = get_pkg_output_path(&self.path);
        debug!("Cleaning {}", &path);
        remove_dir_all(path)?;

        Ok(())
    }
}

#[inline]
pub fn get_pkg_output_path(pkg_path: &Path) -> String {
    super::EXTRACTION_OUTPUT_PATH.to_string()
        + "/"
        + pkg_path.file_stem().unwrap().to_str().unwrap()
}
