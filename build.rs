fn main() {
    let mut path =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    path.push("version.txt");
    std::fs::write(path, std::env::var("CARGO_PKG_VERSION").unwrap()).unwrap();
}
