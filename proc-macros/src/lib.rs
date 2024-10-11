extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use syn::{parse_macro_input, LitStr};

fn get_crate_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
}

fn list_dir_recursively(base_path: &Path) -> Vec<String> {
    let base_path = get_crate_dir().join(base_path);
    let mut module_list = Vec::new();
    for entry in fs::read_dir(base_path).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.is_dir() {
            module_list.extend(list_dir_recursively(&path));
        } else {
            let name = path.to_str().expect("Failed to convert OsString to String").to_string();
            module_list.push(name);
        }
    }
    module_list
}

/// Generate a recursive list of all files in a directory
/// ```
/// use mirams_proc_macros::generate_recursive_dir_list;
/// 
/// const FILES: &'static [&'static str] = &generate_recursive_dir_list!(".");
/// println!("{:?}", FILES);
/// ```
#[proc_macro]
pub fn generate_recursive_dir_list(input: TokenStream) -> TokenStream {
    // Parse the input as a string literal representing the base directory path
    let base_path = parse_macro_input!(input as LitStr).value();

    // Generate the file list recursively
    let path_list = list_dir_recursively(base_path.as_ref());

    // Generate the static array of string slices
    let path_array = quote! {
        [
            #(#path_list),*
        ]
    };

    // Return the output as a TokenStream
    path_array.into()
}

/// Generate a recursive list of pairs of the paths and the file contents
/// for all files in a directory, returning an array of `(&str, &[u8])` tuples
/// 
/// ```
/// use mirams_proc_macros::generate_recursive_dir_content_list;
/// 
/// const FILES: &'static [(&'static str, &'static [u8])] = &generate_recursive_dir_content_list!(".");
/// println!("{:?}", FILES);
/// ```
#[proc_macro]
pub fn generate_recursive_dir_content_list(input: TokenStream) -> TokenStream {
    // Parse the input as a string literal representing the base directory path
    let base_path = parse_macro_input!(input as LitStr).value();

    // Generate the file list recursively
    let path_list = list_dir_recursively(base_path.as_ref());

    // Generate the static array of pairs of path strings and file contents
    let content_array = quote! {
        [
            #(
                (
                    #path_list,
                    std::include_bytes!(#path_list) as &[u8]
                )
            ),*
        ]
    };

    // Return the output as a TokenStream
    content_array.into()
}
