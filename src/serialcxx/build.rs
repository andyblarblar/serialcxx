
fn main() {
    let _build = cxx_build::bridge("src/lib.rs");

    //C bindings for when cxx doesnt support something
    let bindings = cbindgen::Builder::new()
        .with_src("src/bindgenffi.rs")
        .generate()
        .unwrap();
    bindings.write_to_file("generated/bindings.hpp");//TODO figure out how to use this with cmake

    //println!("cargo:rerun-if-changed=src/lib.rs");
}
