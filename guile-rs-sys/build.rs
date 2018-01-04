extern crate bindgen;
extern crate gcc;

use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_path      = PathBuf::from(env::var("OUT_DIR").unwrap());

    let guile_inc_dir = "/usr/include/guile/2.2/";

    gcc::Build::new()
        .include(guile_inc_dir)
        .file(manifest_path.join("wrapper.c"))
        .compile("guile_wrapper");

    // println!("cargo:rustc-link-lib=static=guile_wrapper");


    // println!("cargo:include=/usr/include/guile/2.2/");
    println!("cargo:rustc-link-lib=guile-2.2");

    let bindings = bindgen::Builder::default()
        .header(manifest_path.join("wrapper.c").to_str().unwrap())
        .ctypes_prefix("libc")

        .clang_args(vec![format!("-I{}", guile_inc_dir)])
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");


}
