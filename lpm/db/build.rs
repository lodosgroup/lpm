use std::path::Path;

fn main() {
    let home_path: &'static str = env!("HOME");
    let target_dir = Path::new(&home_path).join(".local/share/min_sqlite3_sys");

    println!("cargo:rustc-link-arg=-Wl,-rpath={}", target_dir.display());
}
