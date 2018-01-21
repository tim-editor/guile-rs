extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_path      = PathBuf::from(env::var("OUT_DIR").unwrap());

    let include_dirs = [
        "/usr/include/guile/2.2",
        "/usr/local/include/guile/2.2",
    ];

    let mut cc = cc::Build::new();

    include_dirs
        .iter()
        .for_each(|s| { cc.include(s); }); // {} required to return ()

    cc
        .file(manifest_path.join("wrapper.c"))
        .compile("guile_wrapper");

    // println!("cargo:rustc-link-lib=static=guile_wrapper");

    // println!("cargo:include=/usr/include/guile/2.2/");
    println!("cargo:rustc-link-lib=guile-2.2");

    let bindings = bindgen::Builder::default()
        .header(manifest_path.join("wrapper.c").to_str().unwrap())
        .ctypes_prefix("libc")

        .clang_args(include_dirs // Add each directory
                        .into_iter()
                        .map(|s| format!("-I{}", s))
                   )
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
