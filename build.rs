
use dioxus_cli_config::DioxusConfig;
use dioxus_cli_config::Platform;
use dioxus_cli::*;
use std::path::Path;

fn main () {
    println!("cargo:rerun-if-changed=frontend");
    let root_crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bin = root_crate_dir.join("frontend");

    // If the frontend directory doesn't exist, we don't need to build it
    // (possibly true for Crates.io uploaded version)
    if !bin.join("Cargo.toml").exists() {
        return;
    }

    let dx = std::env::var_os("CARGO_BIN_FILE_DIOXUS_CLI_dx")
        .expect("Cargo did not supply the dx artifact");

    let _ = std::process::Command::new(dx)
        .current_dir("frontend")
        .args(["build", "--web", "--release", "--skip-assets"])
        .status()
        .expect("failed to run dx");
}

