//! The bindings bridge.

mod bindgenffi;
mod serial;
mod serial_ext;

use serial::*;

#[cxx::bridge(namespace = "serialcxx")]
pub mod ffi {

    pub struct ReadResult {
        /// The error this read produced, if any.
        pub error: SerialError,
        /// The number of bytes read into the buffer.
        pub bytes_read: usize,
    }

    pub enum SerialError {
        /// The operation succeeded.
        NoErr = 0,
        /// The action timed out.
        Timeout,
        /// The operation was interrupted, but did not fail. Can be started again.
        Interrupted,
        /// The port errored while opening or cloning.
        PortIOErr,
        /// Uncategorized error.
        Other,
    }

    pub enum CharSize {
        Five,
        Six,
        Seven,
        Eight,
    }

    pub enum Parity {
        Even,
        Odd,
        None,
    }

    pub enum FlowControl {
        Hardware,
        Software,
        None,
    }

    //The Serial class
    extern "Rust" {

        /// Provides full duplex access to a serial port in a safe manner.
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

        /// Sets the timeout for this port.
        ///
        /// Returns true if the operation succeeded.
        pub fn set_timeout(self: &mut Serial, sec: f32) -> bool;

        /// Sets the character size of this port.
        ///
        /// Returns true if the operation succeeded.
        pub fn set_data_size(self: &mut Serial, bits: CharSize) -> bool;

        /// Sets the baud rate of the port.
        ///
        /// Returns true if the operation succeeded.
        pub fn set_baud_rate(self: &mut Serial, baud: u32) -> bool;

        /// Sets the number of stop bits.
        /// True for two stop bits, false for one.
        ///
        /// Returns true if the operation succeeded.
        pub fn set_stop_bits(self: &mut Serial, two_bits: bool) -> bool;

        /// Sets the parity checking mode.
        ///
        /// Returns true if the operation succeeded.
        pub fn set_parity(self: &mut Serial, mode: Parity) -> bool;

        /// Sets the flow control mode.
        ///
        /// Returns true if the operation succeeded.
        pub fn set_flow_control(self: &mut Serial, mode: FlowControl) -> bool;
    }

    extern "Rust" {
        type SerialListenerBuilder;
        type SerialListener;

        /// Creates a builder to build a reader on this port. This reader will asynchronously read
        /// lines from the port, and perform a callback on each. This reader will inherit all settings from
        /// this port, including any changes after this call.
        ///
        /// This function will throw if the port handle cannot be cloned.
        /// # Usage
        /// In order to build, first call this function, catch the exception, and then use [serialcxx::add_read_callback]
        /// to add the reader callback to this builder. This function is free due to a limitation in the
        /// codegen library used. If this callback is not added, then building will throw.
        pub fn create_listener_builder(self: &Serial) -> Result<Box<SerialListenerBuilder>>;



        /// Attempts to build a listener. This function should be considered to move the builder, and
        /// will throw if the same builder is used twice.
        ///
        /// This function will throw if the callback is not set, or this builder is used twice.
        pub fn build(self: &mut SerialListenerBuilder) -> Result<Box<SerialListener>>;

        /// Gets a pointer to self. Shim to avoid messing with rust::box. Use this to pass this builder
        /// to to the callback adder function.
        ///
        /// Obviously dont free this pointer or things will blow up.
        pub fn self_ptr(self: &mut SerialListenerBuilder) -> *mut SerialListenerBuilder;



        /// Starts the listener thread, calling the callback on each line read from the port.
        ///
        /// This call will lock the read handle to the serialport for as long as the thread is alive.
        /// This means any calls to [Serial::read], [Serial::read_line], or other listeners will block
        /// until this listener dies.
        ///
        /// To end this listener, call [stop] or [SerialListener]'s destructor (they do the same thing).
        ///
        /// # Notes
        /// The listener thread reads in an infinite loop. Each iteration is at most as long as the serial
        /// ports timout configuration. Because of this, listeners can have very poor performance with very low
        /// timeout values.
        pub fn listen(self: & SerialListener);

        /// Stops the listener.
        ///
        /// This should be considered a move of this listener, as any future calls to listen will instantly
        /// complete after this is called. You need to build a new listener to listen again.
        pub fn stop(self: & SerialListener);
    }
}
