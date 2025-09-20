use std::{env, path::PathBuf, str::FromStr};

fn main() {

 println!("cargo::rerun-if-changed=src/capi/api.rs");

let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
        .with_language(cbindgen::Language::C)
        .with_crate(crate_dir)
        .with_include_guard("FT_RUSTBINDINGS_H")
        .generate()
        .expect("Unable to generate rust -> c bindings!")
        .write_to_file("src/capi/header/ft_rustbindings.h");

    let bindings = bindgen::Builder::default()
            .header("./src/capi/header/ft_api.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .//no copy derive
            .generate()
            .expect("Unable to generate c -> rust bindings!");
    let out_path = PathBuf::from_str("./src/capi/cbindings.rs")
            .expect("Project structure incorrect");
    bindings.write_to_file(out_path)
            .expect("Couldn't write bindings!");

//     cc::Build::new()
//         .file("src/capi/ft_string.c")
//         .compile("ft_c_bin");
}