use common::{
    pkg::{MetaDir, PkgDataFromFs},
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

pub trait PkgExtractTasks {
    fn start_extract_task(pkg_path: &Path) -> Result<Self, LpmError<io::Error>>
    where
        Self: Sized;
    fn half_extract(pkg_path: &Path) -> Result<(), LpmError<io::Error>>;
    fn extract_meta_and_program(pkg_path: &Path) -> Result<(), LpmError<io::Error>>;
    fn read_pkg_data(pkg_path: &Path) -> PkgDataFromFs;
    fn cleanup(&self) -> Result<(), LpmError<io::Error>>;
}

impl PkgExtractTasks for PkgDataFromFs {
    fn start_extract_task(pkg_path: &Path) -> Result<Self, LpmError<io::Error>>
    where
        Self: Sized,
    {
        PkgDataFromFs::half_extract(pkg_path)?;
        PkgDataFromFs::extract_meta_and_program(pkg_path)?;
        let pkg_data = PkgDataFromFs::read_pkg_data(pkg_path);

        Ok(pkg_data)
    }

    fn half_extract(pkg_path: &Path) -> Result<(), LpmError<io::Error>> {
        let input_file = File::open(pkg_path)?;
        let mut archive = ar::Archive::new(input_file);

        while let Some(entry) = archive.next_entry() {
            let mut entry = entry?;
            let filename = from_utf8(entry.header().identifier()).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Package has a file with non UTF-8 filename.",
                )
            })?;
            let mut output_path = get_pkg_output_path(pkg_path);

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

    fn extract_meta_and_program(pkg_path: &Path) -> Result<(), LpmError<io::Error>> {
        let pkg_dir = get_pkg_output_path(pkg_path);

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
