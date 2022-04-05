use std::{env, path::Path};

fn main() {
    let home_path = env::var("HOME").expect("HOME environment variable is not set.");
    let target_dir = Path::new(&home_path).join(".local/share/min_sqlite3_sys");

    println!("cargo:rustc-link-arg=-Wl,-rpath={}", target_dir.display());
}
