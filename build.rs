use std::env::var;
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(var("OUT_DIR").unwrap());
    bindgen::builder()
        .header("crossdb/include/crossdb.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap()
        .write_to_file(out.join("./bindings.rs"))
        .unwrap();

    let mut builder = cc::Build::new();
    builder
        .file("crossdb/src/crossdb.c")
        .include("crossdb/include")
        .flag("-lpthread")
        .opt_level(2)
        .static_flag(true);

    // TODO: Potentially unsafe
    builder.cargo_warnings(false);

    #[cfg(target_os = "windows")]
    {
        builder.flag("-lws2_32").compiler("gcc");
    }

    builder.compile("crossdb");
    println!("cargo:rustc-link-lib=static=crossdb");
    println!("cargo:rerun-if-changed=crossdb/");
}
