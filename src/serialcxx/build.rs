
fn main() {
    //C bindings for when cxx doesnt support something
    let bindings = cbindgen::Builder::new()
        .with_src("src/bindgenffi.rs")
        .generate()
        .unwrap();
    bindings.write_to_file("generated/bindings.hpp");

    let _build = cxx_build::bridge("src/lib.rs")
        .include("generated/bindings.hpp")//TODO see if this works
        .compile("serialcxx");

    println!("cargo:rerun-if-changed=src/lib.rs");
}
