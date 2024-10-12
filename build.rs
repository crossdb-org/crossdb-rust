use std::env;
use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::builder()
        .header("crossdb/include/crossdb.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap()
        .write_to_file(out_path.join("./bindings.rs"))
        .unwrap();

    let mut builder = cc::Build::new();
    builder
        .file("crossdb/src/crossdb.c")
        .include("crossdb/include")
        .flag("-fPIC")
        .opt_level(2)
        .static_flag(true)
        .compile("crossdb");
    println!("cargo:rustc-link-lib=static=crossdb");
    println!("cargo:rustc-link-lib=pthread");

    println!("cargo:rerun-if-changed=crossdb/");
}
