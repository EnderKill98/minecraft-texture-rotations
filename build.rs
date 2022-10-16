use std::{env, path::PathBuf};

fn main() {
    // ------------------------------
    // Generate bindings for library

    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // ------------------------------
    // Build and link library
    println!("cargo:rustc-link-lib=pthread");
    cc::Build::new()
        .file("cubiomes/noise.c")
        .file("cubiomes/biome_tree.c")
        .file("cubiomes/layers.c")
        .file("cubiomes/generator.c")
        .file("cubiomes/finders.c")
        .file("cubiomes/util.c")
        .file("cubiomes/quadbase.c")
        .file("extras.c")
        .compile("cubiomes");
}
