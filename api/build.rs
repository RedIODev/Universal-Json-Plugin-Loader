
#![allow(clippy::expect_used, missing_docs, reason = "build script.")]

use std::{path::PathBuf, };
use core::str::FromStr as _;

use bindgen_helpers::{Renamer,
    callbacks::{ItemKind, ParseCallbacks, DeriveInfo},
    rename_enum,
};

#[derive(Debug)]
struct CPrefix;

impl ParseCallbacks for CPrefix {
    fn item_name(&self, item_info: bindgen_helpers::callbacks::ItemInfo) -> Option<String> {
        if matches!(item_info.kind, ItemKind::Type) {
            return Some(format!("C{}", item_info.name));
        }
        None
    }
}

#[derive(Debug)]
struct CloneDerive;

impl ParseCallbacks for CloneDerive {
    fn add_derives(&self, info: &DeriveInfo<'_>) -> Vec<String> {
        if info.name == "CApiVersion" {
            return vec!["Clone".to_owned()];
        }
        vec![]
    }
}

fn main() {
    println!("cargo::rerun-if-changed=src/capi/header/ft_api.h");

    // let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // cbindgen::Builder::new()
    //     .with_language(cbindgen::Language::C)
    //     .with_crate(crate_dir)
    //     .with_include_guard("FT_RUSTBINDINGS_H")
    //     .generate()
    //     .expect("Unable to generate rust -> c bindings!")
    //     .write_to_file("src/capi/header/ft_rustbindings.h");

    let mut renamer = Renamer::new(false);

    rename_enum!(
        renamer,
        "ServiceError" => "ServiceError",
        remove: "^SERVICE_"
    );

    let bindings = bindgen_helpers::Builder::default()
        .use_core()
        .header("./src/capi/header/ft_api.h")
        .parse_callbacks(Box::new(bindgen_helpers::CargoCallbacks::new()))
        .derive_copy(false)
        .default_enum_style(bindgen_helpers::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .parse_callbacks(Box::new(CloneDerive))
        .parse_callbacks(Box::new(renamer))
        .parse_callbacks(Box::new(CPrefix))
        .generate()
        .expect("Unable to generate c -> rust bindings!");
    let out_path = PathBuf::from_str("./src/cbindings.rs").expect("Project structure incorrect");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");

    cc::Build::new()
        .file("src/capi/ft_string.c")
        .compile("ft_c_bin");
}
