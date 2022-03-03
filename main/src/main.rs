use core::{pkg::LodPkg, InstallationTasks};
use std::env;

fn main() {
    if let Some(file) = env::args().nth(1) {
        let mut pkg = LodPkg::new(&file);
        pkg.start_installation().unwrap();
    } else {
        panic!("Missing argument");
    }
}
