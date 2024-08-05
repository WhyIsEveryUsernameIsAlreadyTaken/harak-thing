fn main() {
    println!("cargo::rustc-cfg=DEV_MODE");
    println!("cargo::rustc-check-cfg=cfg(_WIN32)");
    println!("cargo::rustc-check-cfg=cfg(DEV_MODE)");
}
