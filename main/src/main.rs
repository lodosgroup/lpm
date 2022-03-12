use core::{pkg::LodPkg, InstallationTasks};
use std::env;

use ehandle::RuntimeError;

#[cfg(not(target_os = "linux"))]
compile_error!("LodPM can not be built on non-linux operating systems.");

fn main() -> Result<(), RuntimeError> {
    if let Some(file) = env::args().nth(1) {
        let mut pkg = LodPkg::new(&file);
        pkg.start_installation()?;
    } else {
        panic!("Missing argument");
    }

    Ok(())
}

