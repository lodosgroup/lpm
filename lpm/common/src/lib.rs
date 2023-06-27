pub mod lpm_version;
pub mod meta;
pub mod pkg;
pub mod system;
pub mod version;

// re-exports
pub use meta::Files;

use rekuest::Rekuest;
use std::{fs, io, path::Path};

pub trait ParserTasks {
    fn deserialize(path: &str) -> Self;
}

// For non-binary packages
pub const NO_ARCH: &str = "no-arch";

// Supported CPU architectures
#[cfg(target_arch = "x86_64")]
pub const SYSTEM_ARCH: &str = "amd64";
#[cfg(target_arch = "arm")]
pub const SYSTEM_ARCH: &str = "arm";

#[macro_export]
macro_rules! de_required_field {
    ($json: expr, $field: expr) => {
        match $json {
            Some(val) => val,
            None => {
                return Err(format!(
                    "Field '{}' is required and must be provided.",
                    $field
                ))
            }
        }
    };
}

#[macro_export]
macro_rules! some_or_error {
    ($fn: expr, $log: expr, $($args: tt)+) => {
        match $fn {
            Some(val) => val,
            None => panic!("{}", format!($log, $($args)+)),
        }
    };
    ($fn: expr, $log: expr) => {
        match $fn {
            Some(val) => val,
            None => panic!("{}", format!($log)),
        }

    }
}

pub fn download_file(url: &str, output_dir: &Path) -> std::io::Result<()> {
    logger::info!("Downloading from '{url}' into {}", output_dir.display());
    let response = Rekuest::new(url)?.get()?;

    fs::create_dir_all(some_or_error!(
        output_dir.parent(),
        "Failed creating parent directories of {}",
        output_dir.display()
    ))?;

    let mut file = fs::File::create(output_dir)?;
    io::Write::write_all(&mut file, &response.body)?;
    io::Write::flush(&mut file)?;

    Ok(())
}

#[macro_export]
macro_rules! ctx_confirmation_check {
    ($ctx: expr) => {
        if !$ctx.ask_for_confirmation("Do you want to continue?")? {
            std::process::exit(0);
        }
    };
}
