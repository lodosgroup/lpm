use core::{pkg::LodPkg, InstallationTasks};
use std::env;

use ehandle::{RuntimeError, RuntimeErrorKind};

#[cfg(not(target_os = "linux"))]
compile_error!("LodPM can not be built on non-linux operating systems.");

#[cfg(target_os = "linux")]
fn main() -> Result<(), RuntimeError> {
    if let Some(file) = env::args().nth(1) {
        let mut pkg = LodPkg::new(&file);
        pkg.start_installation()?;
    } else {
        panic!("Missing argument");
    }

    Ok(())
}

// handle with pre-build executions
#[cfg(not(target_os = "linux"))]
fn main() -> Result<(), RuntimeError> {
    Err(RuntimeError::new(RuntimeErrorKind::UnsupportedPlatform))
}
