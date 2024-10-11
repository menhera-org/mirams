
pub use mirams_proc_macros::generate_recursive_dir_content_list;
pub use mirams_proc_macros::generate_recursive_dir_list;

use std::sync::OnceLock;
use std::collections::HashMap;

const FRONTEND_FILES: &'static [(&'static str, &'static [u8])] = &generate_recursive_dir_content_list!("./frontend-dist");

static FRONTEND_FILES_MAP: OnceLock<HashMap<&'static str, &'static [u8]>> = OnceLock::new();

pub fn frontend_files() -> &'static HashMap<&'static str, &'static [u8]> {
    FRONTEND_FILES_MAP.get_or_init(|| {
        FRONTEND_FILES.iter().map(|pair| *pair).collect()
    })
}

