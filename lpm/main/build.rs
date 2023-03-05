#[cfg(target_family = "unix")]
fn main() {
    use std::path::Path;

    let home_path: &'static str = env!("HOME");
    let target_dir = Path::new(&home_path).join(".local/share/min_sqlite3_sys");

    println!("cargo:rustc-link-arg=-Wl,-rpath={}", target_dir.display());
}

#[cfg(not(target_family = "unix"))]
fn main() {
    common::log_and_panic!("Lpm can only build on unix systems");
}
