fn main() {
    tauri_build::build();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "macos" {
        // Mach-O adds a leading underscore. Force the linker to pull our
        // definition of Clang's availability helper out of the rlib.
        println!("cargo:rustc-link-arg=-Wl,-u,___isPlatformVersionAtLeast");
    }
}
