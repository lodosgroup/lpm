use std::{
    fs::{self, create_dir_all},
    path::Path,
};

use ehandle::RuntimeError;

pub fn copy_recursively(src: &str, destination: &str) -> Result<(), RuntimeError> {
    create_dir_all(destination.clone())?;

    let src = Path::new(src);
    let destination = Path::new(destination);

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_recursively(
                entry.path().to_str().unwrap(),
                destination.join(entry.file_name()).to_str().unwrap(),
            )?;
        } else {
            fs::copy(
                entry.path(),
                destination.join(entry.file_name()).to_str().unwrap(),
            )?;
        }
    }

    Ok(())
}
