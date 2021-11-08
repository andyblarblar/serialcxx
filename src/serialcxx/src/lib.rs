mod serial;
mod bindgenffi;

use serial::*;

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type Serial;

        fn open_port(path: &str, baud: u32) -> Result<Box<Serial>>;
    }
}

