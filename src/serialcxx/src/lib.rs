mod bindgenffi;
mod serial;

use serial::*;

#[cxx::bridge(namespace="serialcxx")]
pub mod ffi {

    pub struct ReadResult {
        /// The error this read produced, if any.
        pub error: SerialError,
        /// The number of bytes read into the buffer.
        pub bytes_read: usize
    }

    pub enum SerialError {
        /// The operation succeeded.
        NoErr = 0,
        /// The operation was interrupted, but did not fail. Can be started again.
        Interrupted,
        /// The port errored while opening or cloning.
        PortIOErr,
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

        /// Attempts to write the entire string to the serial device.
        ///
        /// Errors
        /// ------
        ///
        /// - Interrupted - The device transfer was interrupted. You may retry this transfer.
        /// - Other - Any other kind of device failure, such as a disconnect.
        fn write_str(self: &mut Serial, data: &CxxString) -> SerialError;

        /// Attempts to read the remaining serial device's buffer, up to the size of the passed slice.
        /// Return struct includes the number of bytes read into this buffer.
        ///
        /// Errors
        /// ------
        ///
        /// - Interrupted - The device transfer was interrupted. You may retry this transfer.
        /// - Other - Any other kind of device failure, such as a disconnect.
        fn read(self: &mut Serial, read_buff: &mut [u8]) -> ReadResult;

        /// Attempts to read a line from the buffer.
        ///
        /// A line is defined by a terminating \n or \r\n, neither of which will be present in the
        /// returned string.
        ///
        /// Errors
        /// ------
        ///
        /// - Interrupted - The device transfer was interrupted. You may retry this transfer.
        /// - PortIOErr - The port failed to clone handle for reads.
        /// - Other - Any other kind of device failure, such as a disconnect.
        fn read_line(self: &mut Serial, read_buff: Pin<&mut CxxString>) -> ReadResult;

        /// Attempts to open the serial device at path, using the specified baud rate.
        /// Defaults to a timeout of 99999 seconds.
        fn open_port(path: &str, baud: u32) -> Result<Box<Serial>>;
    }
}