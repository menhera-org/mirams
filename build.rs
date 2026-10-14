use std::path::Path;

// Keep the build dependency that enables vendored OpenSSL for the dx build.
use openssl_sys as _;

fn copy_dir(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn main() {
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
    let bundle_dir =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("frontend-bundle");
    if bundle_dir.exists() {
        std::fs::remove_dir_all(&bundle_dir).expect("failed to clear frontend bundle directory");
    }

    let dx_status = std::process::Command::new(dx)
        .current_dir(&bin)
        .args(["bundle", "--web", "--release", "--out-dir"])
        .arg(&bundle_dir)
        .status()
        .expect("failed to run dx");

    if !dx_status.success() {
        panic!("dx bundle failed: {dx_status}");
    }

    let public_dir = bundle_dir.join("public");
    assert!(
        public_dir.join("index.html").is_file(),
        "dx bundle did not produce public/index.html"
    );
    let frontend_dist = root_crate_dir.join("frontend-dist");
    if frontend_dist.exists() {
        std::fs::remove_dir_all(&frontend_dist).expect("failed to clear frontend-dist");
    }
    copy_dir(&public_dir, &frontend_dist).expect("failed to copy frontend bundle to frontend-dist");
}
