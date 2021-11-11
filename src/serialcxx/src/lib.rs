mod bindgenffi;
mod serial;

use serial::*;

#[cxx::bridge(namespace="serialcxx")]
pub mod ffi {

    pub enum SerialError {
        /// The operation succeeded.
        NoErr = 0,
        /// The operation was interrupted, but did not fail. Can be started again.
        Interrupted,
        /// Uncategorized error.
        Other
    }

    extern "Rust" {
        type Serial;

        /// Attempts to write the entire buffer of bytes to the serial device.
        ///
        /// Errors
        /// ------
        ///
        /// - Interrupted - The device transfer was interrupted. You may retry this transfer.
        /// - Other - Any other kind of device failure, such as a disconnect.
        fn write(self: &mut Serial, data: &[u8]) -> SerialError;
        fn open_port(path: &str, baud: u32) -> Result<Box<Serial>>;
    }
}