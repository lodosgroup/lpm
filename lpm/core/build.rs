#[cfg(target_family = "unix")]
fn main() {
    use std::path::Path;

    let home_path: &'static str = env!("HOME");
    let sqlite_so = Path::new(&home_path).join(".local/share/min_sqlite3_sys");
    let lz4_so = Path::new(&home_path).join(".local/share/tiny_lz4_decoder_sys");

    #[cfg(not(target_os = "macos"))]
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", sqlite_so.display());
    #[cfg(not(target_os = "macos"))]
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lz4_so.display());

    #[cfg(target_os = "macos")]
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,$loader_path/{}",
        sqlite_so.display()
    );
    #[cfg(target_os = "macos")]
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,$loader_path/{}",
        lz4_so.display()
    );
}

#[cfg(not(target_family = "unix"))]
fn main() {
    common::log_and_panic!("Lpm can only build on unix systems");
}