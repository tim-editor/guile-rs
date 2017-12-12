extern crate bindgen;
extern crate gcc;

use std::env;
use std::path::PathBuf;

fn main() {
    let guile_inc_dir = "/usr/include/guile/2.2";

    gcc::Build::new()
        .include(guile_inc_dir)
        .file("wrapper.c")
        .compile("guile_wrapper");

    // println!("cargo:rustc-link-lib=static=guile_wrapper");


    // println!("cargo:include=/usr/include/guile/2.2/");
    println!("cargo:rustc-link-lib=guile-2.2");

    let bindings = bindgen::Builder::default()
        .header("wrapper.c")
        .ctypes_prefix("libc")

        .clang_args(vec![format!("-I{}", guile_inc_dir)])
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");


}
