
pub use mirams_proc_macros::generate_recursive_dir_content_list;
pub use mirams_proc_macros::generate_recursive_dir_list;

use std::path::Path;
use std::sync::OnceLock;
use std::collections::HashMap;

/// If this is not "/favicon.ico", /favicon.ico will be redirected to this path.
pub const ICON_PATH: &'static str = "/icon.svg";

const CRATE_ROOT: &'static str = env!("CARGO_MANIFEST_DIR");
const FRONTEND_FILES: &'static [(&'static str, &'static [u8])] = &generate_recursive_dir_content_list!("./frontend-dist");

static FRONTEND_FILES_MAP: OnceLock<HashMap<String, &'static [u8]>> = OnceLock::new();

pub fn frontend_files() -> &'static HashMap<String, &'static [u8]> {
    let frontend_root = Path::new(CRATE_ROOT).join("frontend-dist");
    FRONTEND_FILES_MAP.get_or_init(|| {
        FRONTEND_FILES.iter().map(|pair| {
            let path = pathdiff::diff_paths(pair.0, &frontend_root).unwrap();
            let content = pair.1;
            (path.to_string_lossy().to_string(), content)
        }).collect()
    })
}

const TYPES_BY_EXT: &'static [(&'static str, &'static str)] = &[
    ("html", "text/html"),
    ("css", "text/css"),
    ("js", "text/javascript"),
    ("json", "application/json"),
    ("svg", "image/svg+xml"),
    ("webmanifest", "application/manifest+json"),
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("ico", "image/x-icon"),
    ("br", "application/x-brotli"),
    ("wasm", "application/wasm"),
    ("map", "application/json"),
    ("txt", "text/plain"),
    ("xml", "application/xml"),
    ("md", "text/markdown"),
];

static TYPES_BY_EXT_MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

pub fn types_by_ext() -> &'static HashMap<&'static str, &'static str> {
    TYPES_BY_EXT_MAP.get_or_init(|| {
        TYPES_BY_EXT.iter().map(|pair| *pair).collect()
    })
}

