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

/// Just a temporary value, which will once https://github.com/lodosgroup/lpm/pull/68 is merged.
#[cfg(not(any(target_arch = "x86_64", target_arch = "arm")))]
pub const SYSTEM_ARCH: &str = "temporary-val";

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
            None => {
                logger::error!("{}", format!($log, $($args)+));
                std::process::exit(101);
            },
        }
    };
    ($fn: expr, $log: expr) => {
        match $fn {
            Some(val) => val,
            None => {
                logger::error!("{}", format!($log));
                std::process::exit(101);
            },
        }

    }
}

pub fn download_file(url: &str, output_path: &Path) -> std::io::Result<()> {
    let pkg_filename = output_path.file_name().unwrap();
    // TODO
    // We should check if user wants to force re-downloading.
    if output_path.exists() {
        logger::info!(
            "Skipping package download for {:?}; already exists: '{}'",
            pkg_filename,
            output_path.display()
        );

        return Ok(());
    }

    logger::info!(
        "Downloading {:?} into '{}'",
        pkg_filename,
        output_path.display()
    );
    let response = Rekuest::new(url)?.get()?;

    fs::create_dir_all(some_or_error!(
        output_path.parent(),
        "Failed creating parent directories of '{}'",
        output_path.display()
    ))?;

    let mut file = fs::File::create(output_path)?;
    io::Write::write_all(&mut file, &response.body)?;
    io::Write::flush(&mut file)?;

    logger::debug!("Download of {:?} was successful", pkg_filename);

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
