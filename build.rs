
use std::path::Path;

fn main () {
    println!("cargo:rerun-if-changed=frontend");
    println!("cargo:rerun-if-changed=frontend-dist");
    let root_crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bin = root_crate_dir.join("frontend");
    // If the frontend directory doesn't exist, we don't need to build it
    // (possibly true for Crates.io uploaded version)
    if !bin.join("Cargo.toml").exists() {
        return;
    }

    let dx = std::env::var_os("CARGO_BIN_FILE_DIOXUS_CLI_dx")
        .expect("Cargo did not supply the dx artifact");

    let dx_status = std::process::Command::new(dx)
        .current_dir(&bin)
        .args(["build", "--web", "--release"])
        .status()
        .expect("failed to run dx");

    if !dx_status.success() {
        panic!("dx failed to run");
    }
}

