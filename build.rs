fn write(file: &str, data: String) {
    let mut path =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    path.push(file);
    std::fs::write(path, data).unwrap();
}

fn main() {
    write("version.txt", std::env::var("CARGO_PKG_VERSION").unwrap());
    write("arch.txt", std::env::var("CARGO_CFG_TARGET_ARCH").unwrap());
    write("os.txt", std::env::var("CARGO_CFG_TARGET_OS").unwrap());
}
