extern crate bindgen;
extern crate cmake;

use std::fs::File;
use std::io::Write;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    if !Path::new("proj.4/.git").exists() {
        let _ = Command::new("git").args(&["submodule", "update", "--init"])
            .status();
    }
    let mut cfg = cmake::Config::new("proj.4");

    let out_dir = env::var("OUT_DIR").unwrap();
    let proj_lib_dir = Path::new(&out_dir).join("lib");

    // blow away the rpath and use the absolute path the dylib in the linked artifact
    cfg.define("CMAKE_INSTALL_NAME_DIR", "${CMAKE_INSTALL_PREFIX}/lib");

    cfg.build();

    // Tell cargo to tell rustc to link the system proj
    // shared library.
    println!("cargo:rustc-link-lib=dylib=proj");
    println!(
        r"cargo:rustc-link-search={}",
        proj_lib_dir.to_str().unwrap()
    );

    // create the wrapper header
    let wrapper_path = Path::new(&out_dir).join("include/wrapper.h");
    let mut f = File::create(&wrapper_path).unwrap();
    f.write_all(b"#include \"proj.h\"").unwrap();

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .trust_clang_mangling(false)
        .blacklist_type("max_align_t")
        // The input header we would like to generate
        // bindings for.
        .header(wrapper_path.to_str().unwrap())
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}