use std::ffi::{c_void, CStr, CString};
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

/// The Rust side of the serial facade.
///
/// # Safety
/// This library is designed to prevent improper use at runtime, something C++ is poor at.
/// In particular, it is threadsafe, meaning that concurrent calls to reads/writes will block until
/// one thread finishes. Similarly, async reads through listeners prevent all other reads while alive,
/// preventing accidental reads from the main thread. In general, this libraries priorities are
/// 1) safety 2) ergonomics 3) performence.
///
/// # Implementation
/// This object contains two separate handles to the same serial port. One is for reading, and one
/// is for writing. Each of these ports sit behind a Mutex. This means that this Serial object is
/// thread safe from the C++ side, at the cost of a lock acquisition on each read and write call.
/// This is also what allows us to share the reader handle with the listener, while ensuring only
/// the listener can read from the port while alive.
/// The reader has two mutexes, as it needs to allow for threaded access to the reader itself, as well
/// as threaded access to the raw handle itself, as the settings of all handles can be changed across
/// Threads. These mutexes should rarely be contested, so the performance hit shouldn't be too bad.
///
/// # Listeners
/// Listeners allow for full duplex communication without needing to mess with threads in C++.
/// When you spawn a listener, *it will be the only thread allowed to read from the port for its lifetime*.
/// This means that calls to any other read function (not including settings functions but including other listeners) will block
/// until the thread dies. Once the thread dies, there will be a race on the mutex. It is for this reason
/// that there should be no more than one listener alive at once.
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
                ErrorKind::TimedOut => SerialError::Timeout,
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
                ErrorKind::TimedOut => SerialError::Timeout,
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
                ErrorKind::TimedOut => ReadResult {
                    error: SerialError::Timeout,
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

        match read_num {
            Ok(bytes_read) => {
                //Copy rust string to C++, removing newline
                if bytes_read > 0 {
                    read_buff.push_str(&rust_buff[..rust_buff.len() - 1])
                };

                ReadResult {
                    error: SerialError::NoErr,
                    bytes_read,
                }
            }
            Err(err) => match err.kind() {
                ErrorKind::Interrupted => ReadResult {
                    error: SerialError::Interrupted,
                    bytes_read: 0,
                },
                ErrorKind::TimedOut => ReadResult {
                    error: SerialError::Timeout,
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
    /// lines from the port, and perform a callback on each. This reader will inherit all settings from
    /// this port, including any changes after this call.
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
    /// Token used to kill the thread.
    cts: CancellationTokenSource,
}

impl SerialListener {
    /// Starts the listener thread, calling the callback on each line read from the port.
    ///
    /// This call will lock the read handle to the serialport for as long as the thread is alive.
    /// This means any calls to [Serial::read], [Serial::read_line], or other listeners will block
    /// until this listener dies.
    ///
    /// To end this listener, call [stop] or [SerialListener]'s destructor (they do the same thing).
    pub fn listen(&self) {
        //The cancellation token is the only way we have to kill the listener thread.
        let token = self.cts.token().clone();
        let reader = self.reader.clone();
        let callback = self.callback;

        //Lock the mutex to prevent a race before this thread spawns
        let _out_lock = self.reader.lock();

        std::thread::spawn(move || {
            let (user_data, callback) = callback;
            //Lock the reader while this listener is active
            let mut reader = reader.lock();

            while !token.is_canceled() {
                let mut str_buf = String::with_capacity(40);
                let read_num = reader.read_line(&mut str_buf);

                if let Ok(num) = read_num {
                    if num > 0 {
                        //Strip newline and add nullchar
                        let c_str = CString::new(&str_buf[..str_buf.len() - 1]).unwrap(); //TODO handle unwrap

                        unsafe {
                            //Safe only if callback does not store a reference to the string, which it does not own.
                            callback(user_data.0, c_str.as_ptr(), num);
                            println!("out of callback")
                        }
                    }
                }
            }

            println!("exiting reader") //TODO change to log facade
        });
        //Thread detaches here
    }

    /// Stops the listener.
    ///
    /// This should be considered a move of this listener, as any future calls to listen will instantly
    /// complete after this is called. You need to build a new listener to listen again.
    pub fn stop(&self) {
        self.cts.cancel();
    }
}

impl Drop for SerialListener {
    fn drop(&mut self) {
        self.cts.cancel() //Cancel the detached thread. The token will be kept alive by the thread, so this doesn't create a dangling pointer.
    }
}
