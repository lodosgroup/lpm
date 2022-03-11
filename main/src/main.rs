use core::{pkg::LodPkg, InstallationTasks};
use std::env;

use ehandle::RuntimeError;

fn main() -> Result<(), RuntimeError> {
    if let Some(file) = env::args().nth(1) {
        let mut pkg = LodPkg::new(&file);
        pkg.start_installation()?;
    } else {
        panic!("Missing argument");
    }

    Ok(())
}
