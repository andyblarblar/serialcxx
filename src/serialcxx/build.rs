fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/bindgenffi.rs");

    let out_str = std::env::var("OUT_DIR").unwrap();
    let out_dir = std::path::Path::new(out_str.as_str());

    //Build cxx bridge normally
    let _build = cxx_build::bridge("src/lib.rs")
        .flag_if_supported("std=c++11")
        .compile("serialcxx");

    //C bindings for when cxx doesnt support something
    let bindings = cbindgen::Builder::new()
        .with_src("src/bindgenffi.rs")
        .with_namespace("serialcxx")
        .with_include("serialcxx/src/lib.rs.h") //Include cxx generated headers to allow for clean includes
        .generate()
        .unwrap();

    //write at root to allow for including with #include "serialcxx/serialcxx.hpp"
    bindings.write_to_file(out_dir.join("../../../../cxxbridge/serialcxx/serialcxx.hpp"));

    println!(
        "Wrote cbindgen header to {:?}",
        out_dir.join("../../../../cxxbridge/serialcxx/serialcxx.hpp")
    );
}
