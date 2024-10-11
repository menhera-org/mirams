
use dioxus_cli_config::DioxusConfig;
use dioxus_cli_config::Platform;
use dioxus_cli::*;
use std::path::Path;

fn main () {
    println!("cargo:rerun-if-changed=frontend");
    let root_crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bin = root_crate_dir.join("frontend");
    let _config = DioxusConfig::load(Some(bin.clone())).unwrap().unwrap_or(DioxusConfig::default());
    let opts = cli::cfg::ConfigOptsBuild {
        release: true,
        force_debug: false,
        verbose: false,
        skip_assets: false,
        client_feature: "web".to_string(),
        server_feature: "server".to_string(),
        example: None,
        profile: None,
        platform: Some(Platform::Web),
        features: None,
        target: None,
        cargo_args: vec![],
    };
    let build = cli::build::Build {
        build: opts,
    };
    build.build(Some(bin.clone()), None, None).unwrap();
}

