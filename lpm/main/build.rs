#[cfg(target_family = "unix")]
fn main() {
    use std::path::Path;

    let home_path: &'static str = env!("HOME");
    let sqlite_so = Path::new(&home_path).join(".local/share/min_sqlite3_sys");
    let lz4_so = Path::new(&home_path).join(".local/share/tiny_lz4_decoder_sys");

    println!(
        "cargo:rustc-link-arg=-Wl,-rpath={},-rpath={}",
        sqlite_so.display(),
        lz4_so.display()
    );
}

#[cfg(not(target_family = "unix"))]
fn main() {
    common::log_and_panic!("Lpm can only build on unix systems");
}
