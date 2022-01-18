use std::ffi::c_void;
use std::io::BufRead;
use std::io::BufReader;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::os::raw::c_char;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use cancellation::CancellationTokenSource;
use cxx::CxxString;
use serialport::{DataBits, Error, Result, SerialPort, StopBits};

use crate::ffi::{CharSize, FlowControl, Parity, ReadResult, SerialError};
use crate::serial_ext::{CVoidSend, SerialPortReader};

pub(crate) type Mutex<T> = parking_lot::Mutex<T>;
pub(crate) type MutexGuard<'a, T> = parking_lot::MutexGuard<'a, T>;

//TODO remove warnings about settings not syncing, as they do now.

/// The Rust side of the serial facade.
///
/// # Implementation
/// This object contains two separate handles to the same serial port. One is for reading, and one
/// is for writing. Each of these ports sit behind a Mutex. This means that this Serial object is
/// thread safe from the C++ side, at the cost of a lock acquisition on each read and write call.
/// This is also what allows us to share the reader handle with the listener, while also allowing for
/// reads from the main handle with the listener going. Only one of the ports will get the data, but
/// at least that doesn't break the internal state of the serial port implementation (platform specific).
pub struct Serial {
    write_handle: Mutex<Box<dyn SerialPort>>,
    /// Shared mutex over a reader (shared between main and listener threads) that houses a shared mutex to a handle (Shared to allow for changing settings across all readers).
    read_handle: Arc<Mutex<BufReader<SerialPortReader>>>, //A handle wrapped in a bufreader to allow for using read_line.
    /// Same shared mutex to handle as is inside of [read_handle].
    read_settings_handle: Arc<Mutex<Box<dyn SerialPort>>>, //A reference to the handle above, but not wrapped to allow for changing settings.
}

impl Serial {
    pub fn new(path: &str, baud: u32) -> Result<Serial> {
        //Create two handles, one for reading, and one for writing.
        let raw_port = serialport::new(path, baud)
            .timeout(Duration::from_secs(99999))
            .open()?;

        //Create shared handle
        let port_clone = Arc::from(Mutex::from(raw_port.try_clone()?));

        //Clone that handle to allow for access as a dyn SerialPort for settings changes
        let port_reader_settings = port_clone.clone();
        //Move og handle into a wrapper object that impls Read by delegating to SerialPorts impl, bypassing Rust's lack of Trait casting.
        let port_reader = SerialPortReader::new(port_clone);

        Ok(Serial {
            write_handle: Mutex::new(raw_port),
            read_handle: Arc::new(Mutex::new(BufReader::new(port_reader))),
            read_settings_handle: port_reader_settings,
        })
    }

    /// Locks both mutexes, and returns their handles.
    /// This is used to sync settings between the read and write handles.
    /// This shouldn't be a performance issue, as users should not be changing settings frequently.
    fn lock_both_handles(
        &mut self,
    ) -> (
        MutexGuard<'_, Box<dyn SerialPort>>,
        MutexGuard<'_, Box<dyn SerialPort>>,
    ) {
        let read_settings_lock = self.read_settings_handle.lock();
        let write_lock = self.write_handle.lock();

        (read_settings_lock, write_lock)
    }

    /// Sets the timeout for this port.
    ///
    /// Returns true if the operation succeeded.
    ///
    /// Note that settings changes will not propagate between the serial port and any open readers
    /// or other clones. Consider configuring the port before cloning.
    pub fn set_timeout(&mut self, sec: f32) -> bool {
        let (mut read_handle, mut write_handle) = self.lock_both_handles();

        let read_res = read_handle
            .set_timeout(Duration::from_secs_f32(sec))
            .is_ok();

        let write_res = write_handle
            .set_timeout(Duration::from_secs_f32(sec))
            .is_ok();

        read_res && write_res
    }

    /// Sets the character size of this port.
    ///
    /// Returns true if the operation succeeded.
    ///
    /// Note that settings changes will not propagate between the serial port and any open readers
    /// or other clones. Consider configuring the port before cloning.
    pub fn set_data_size(&mut self, bits: CharSize) -> bool {
        let (mut read_handle, mut write_handle) = self.lock_both_handles();

        let read_res = read_handle
            .set_data_bits(match bits {
                CharSize::Five => DataBits::Five,
                CharSize::Six => DataBits::Six,
                CharSize::Seven => DataBits::Seven,
                CharSize::Eight => DataBits::Eight,
                _ => DataBits::Eight,
            })
            .is_ok();

        let write_res = write_handle
            .set_data_bits(match bits {
                CharSize::Five => DataBits::Five,
                CharSize::Six => DataBits::Six,
                CharSize::Seven => DataBits::Seven,
                CharSize::Eight => DataBits::Eight,
                _ => DataBits::Eight,
            })
            .is_ok();

        read_res && write_res
    }

    /// Sets the baud rate of the port.
    ///
    /// Returns true if the operation succeeded.
    ///
    /// Note that settings changes will not propagate between the serial port and any open readers
    /// or other clones. Consider configuring the port before cloning.
    pub fn set_baud_rate(&mut self, baud: u32) -> bool {
        let (mut read_handle, mut write_handle) = self.lock_both_handles();

        let read_res = read_handle.set_baud_rate(baud).is_ok();

        let write_res = write_handle.set_baud_rate(baud).is_ok();

        read_res && write_res
    }

    /// Sets the number of stop bits.
    /// True for two stop bits, false for one.
    ///
    /// Returns true if the operation succeeded.
    ///
    /// Note that settings changes will not propagate between the serial port and any open readers
    /// or other clones. Consider configuring the port before cloning.
    pub fn set_stop_bits(&mut self, two_bits: bool) -> bool {
        let (mut read_handle, mut write_handle) = self.lock_both_handles();

        let read_res = read_handle
            .set_stop_bits(if two_bits {
                StopBits::Two
            } else {
                StopBits::One
            })
            .is_ok();

        let write_res = write_handle
            .set_stop_bits(if two_bits {
                StopBits::Two
            } else {
                StopBits::One
            })
            .is_ok();

        read_res && write_res
    }

    /// Sets the parity checking mode.
    ///
    /// Returns true if the operation succeeded.
    ///
    /// Note that settings changes will not propagate between the serial port and any open readers
    /// or other clones. Consider configuring the port before cloning.
    pub fn set_parity(&mut self, mode: Parity) -> bool {
        let (mut read_handle, mut write_handle) = self.lock_both_handles();

        let read_res = read_handle
            .set_parity(match mode {
                Parity::Even => serialport::Parity::Even,
                Parity::Odd => serialport::Parity::Odd,
                Parity::None => serialport::Parity::None,
                _ => serialport::Parity::None,
            })
            .is_ok();

        let write_res = write_handle
            .set_parity(match mode {
                Parity::Even => serialport::Parity::Even,
                Parity::Odd => serialport::Parity::Odd,
                Parity::None => serialport::Parity::None,
                _ => serialport::Parity::None,
            })
            .is_ok();

        read_res && write_res
    }

    /// Sets the flow control mode.
    ///
    /// Returns true if the operation succeeded.
    ///
    /// Note that settings changes will not propagate between the serial port and any open readers
    /// or other clones. Consider configuring the port before cloning.
    pub fn set_flow_control(&mut self, mode: FlowControl) -> bool {
        let (mut read_handle, mut write_handle) = self.lock_both_handles();

        let read_res = read_handle
            .set_flow_control(match mode {
                FlowControl::Hardware => serialport::FlowControl::Hardware,
                FlowControl::Software => serialport::FlowControl::Software,
                FlowControl::None => serialport::FlowControl::None,
                _ => serialport::FlowControl::None,
            })
            .is_ok();

        let write_res = write_handle
            .set_flow_control(match mode {
                FlowControl::Hardware => serialport::FlowControl::Hardware,
                FlowControl::Software => serialport::FlowControl::Software,
                FlowControl::None => serialport::FlowControl::None,
                _ => serialport::FlowControl::None,
            })
            .is_ok();

        read_res && write_res
    }

    /// Attempts to write the entire buffer of bytes to the serial device.
    ///
    /// Errors
    /// ------
    ///
    /// - Interrupted - The device transfer was interrupted. You may retry this transfer.
    /// - Other - Any other kind of device failure, such as a disconnect.
    pub fn write(&mut self, data: &[u8]) -> SerialError {
        let mut write_handle = self.write_handle.lock();
        let res = write_handle.write_all(data);

        match res {
            Ok(_) => SerialError::NoErr,
            Err(err) => match err.kind() {
                ErrorKind::Interrupted => SerialError::Interrupted,
                _ => SerialError::Other,
            },
        }
    }

    /// Attempts to write the entire string to the serial device.
    ///
    /// Errors
    /// ------
    ///
    /// - Interrupted - The device transfer was interrupted. You may retry this transfer.
    /// - Other - Any other kind of device failure, such as a disconnect.
    pub fn write_str(&mut self, data: &CxxString) -> SerialError {
        let mut write_handle = self.write_handle.lock();
        let res = write_handle.write_all(data.as_bytes());

        match res {
            Ok(_) => SerialError::NoErr,
            Err(err) => match err.kind() {
                ErrorKind::Interrupted => SerialError::Interrupted,
                _ => SerialError::Other,
            },
        }
    }

    /// Attempts to read the remaining serial device's buffer, up to the size of the passed slice.
    /// Return struct includes the number of bytes read into this buffer. This function does not
    /// block if the serial buffer is read to completion.
    ///
    /// Errors
    /// ------
    ///
    /// - Interrupted - The device transfer was interrupted. You may retry this transfer.
    /// - Other - Any other kind of device failure, such as a disconnect.
    pub fn read(&mut self, read_buff: &mut [u8]) -> ReadResult {
        let mut read_handle = self.read_handle.lock();
        let read_num = read_handle.read(read_buff);

        match read_num {
            Ok(bytes_read) => ReadResult {
                error: SerialError::NoErr,
                bytes_read,
            },
            Err(err) => match err.kind() {
                ErrorKind::Interrupted => ReadResult {
                    error: SerialError::Interrupted,
                    bytes_read: 0,
                },
                _ => ReadResult {
                    error: SerialError::Other,
                    bytes_read: 0,
                },
            },
        }
    }

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
    pub fn read_line(&mut self, read_buff: Pin<&mut CxxString>) -> ReadResult {
        let mut read_handle = self.read_handle.lock();
        let mut rust_buff = String::new();

        let read_num = read_handle.read_line(&mut rust_buff);

        //Copy rust string to C++. Use lines iter to remove newline. This can prob be more efficient.
        read_buff.push_str(rust_buff.lines().take(1).collect::<String>().as_str());

        match read_num {
            Ok(bytes_read) => ReadResult {
                error: SerialError::NoErr,
                bytes_read,
            },
            Err(err) => match err.kind() {
                ErrorKind::Interrupted => ReadResult {
                    error: SerialError::Interrupted,
                    bytes_read: 0,
                },
                _ => ReadResult {
                    error: SerialError::Other,
                    bytes_read: 0,
                },
            },
        }
    }

    /// Creates a builder to build a reader on this port. This reader will asynchronously read
    /// lines from the port, and perform a callback on each. The settings this reader will use will
    /// be the same as this serial port *At the time of this function call*, and will not reflect
    /// future updates.
    ///
    /// This function will throw if the port handle cannot be cloned.
    /// # Usage
    /// In order to build, first call this function, catch the exception, and then use [serialcxx::add_read_callback]
    /// to add the reader callback to this builder. This function is free due to a limitation in the
    /// codegen library used. If this callback is not added, then building will throw.
    pub fn create_listener_builder(&self) -> Result<Box<SerialListenerBuilder>> {
        let clone = self.read_handle.clone();

        Ok(Box::from(SerialListenerBuilder {
            reader: Some(clone),
            callback: None,
        }))
    }
}

/// Attempts to open the serial device at path, using the specified baud rate.
/// Defaults to a timeout of 99999 seconds.
pub fn open_port(path: &str, baud: u32) -> Result<Box<Serial>> {
    Ok(Box::from(Serial::new(path, baud)?))
}

pub struct SerialListenerBuilder {
    pub reader: Option<Arc<Mutex<BufReader<SerialPortReader>>>>, //This is optional as it allows us to 'move' into the listener without move available in cxx.
    pub callback: Option<(
        *mut c_void,
        unsafe extern "C" fn(user_data: *mut c_void, string_read: *const c_char, str_size: usize),
    )>,
}

impl SerialListenerBuilder {
    /// Attempts to build a listener. This function should be considered to move the builder, and
    /// will throw if the same builder is used twice.
    ///
    /// This function will throw if the callback is not set, or this builder is used twice.
    pub fn build(&mut self) -> Result<Box<SerialListener>> {
        if self.callback.is_none() {
            Err(Error::new(
                serialport::ErrorKind::InvalidInput,
                "No callback provided to reader builder.",
            ))
        } else if self.reader.is_none() {
            Err(Error::new(
                serialport::ErrorKind::InvalidInput,
                "Attempting to reuse spent builder. Please make another instead.",
            ))
        } else {
            let callb = self.callback.unwrap();
            Ok(Box::from(SerialListener {
                callback: (CVoidSend(callb.0), callb.1),
                reader: self.reader.take().unwrap(), //Note the take-foo to avoid a move
                cts: CancellationTokenSource::new(),
            }))
        }
    }

    /// Gets a pointer to self. Shim to avoid messing with rust::box. Use this to pass this builder
    /// to to the callback adder function.
    ///
    /// Obviously dont free this pointer or things will blow up.
    pub fn self_ptr(&mut self) -> *mut SerialListenerBuilder {
        self as *mut SerialListenerBuilder
    }
}

pub struct SerialListener {
    reader: Arc<Mutex<BufReader<SerialPortReader>>>,
    callback: (
        CVoidSend,
        unsafe extern "C" fn(user_data: *mut c_void, string_read: *const c_char, str_size: usize),
    ),
    cts: CancellationTokenSource,
}

impl SerialListener {
    fn listen(&self) {
        //The cancellation token is the only way we have to kill the listener thread.
        let token = self.cts.token();
        let reader = self.reader.clone();
        let callback = self.callback.clone();

        let thread = std::thread::spawn(move || {
            let (user_data, callback) = callback;
        }); //TODO impl
    }
}
