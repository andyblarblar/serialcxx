mod bindgenffi;
mod serial;

use serial::*;

#[cxx::bridge(namespace="serialcxx")]
mod ffi {
    extern "Rust" {
        type Serial;

        fn open_port(path: &str, baud: u32) -> Result<Box<Serial>>;
    }
}
