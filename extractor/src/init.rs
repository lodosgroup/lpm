use ar::Archive;
use std::fs::{create_dir_all, File};
use std::io::copy;
use std::path::Path;
use std::str::from_utf8;

use crate::EXTRACTION_OUTPUT_PATH;

pub fn extract_half(file_path: &Path) -> Result<(), std::io::Error> {
    let input_file = File::open(file_path).expect("Package could not opened.");
    let mut archive = Archive::new(input_file);

    while let Some(entry) = archive.next_entry() {
        let mut entry = entry.expect("Failed on parsing archive entry.");
        let filename = from_utf8(entry.header().identifier())
            .expect("Package has a file that has non-utf8 name.");
        let mut output_path =
            EXTRACTION_OUTPUT_PATH.to_string() + file_path.file_stem().unwrap().to_str().unwrap();

        create_dir_all(output_path.clone())?;

        output_path += "/";
        output_path += filename;

        let output_path = Path::new(&output_path).to_path_buf();
        let mut output_file = File::create(&output_path)?;
        copy(&mut entry, &mut output_file).expect(&format!("Failed to copy {:?}.", output_path));
    }

    Ok(())
}
