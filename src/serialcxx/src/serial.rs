use crate::ffi::{ReadResult, SerialError};
use cxx::CxxString;
use serialport::{Result, SerialPort};
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::pin::Pin;
use std::time::Duration;

pub struct Serial {
    raw_port: Box<dyn SerialPort>
}

impl Serial {
    pub fn new(path: &str, baud: u32) -> Result<Serial> {
        let raw_port = serialport::new(path, baud)
            .timeout(Duration::from_secs(99999))
            .open()?;

        Ok(Serial {
            raw_port,
        })
    }

    /// Attempts to write the entire buffer of bytes to the serial device.
    ///
    /// Errors
    /// ------
    ///
    /// - Interrupted - The device transfer was interrupted. You may retry this transfer.
    /// - Other - Any other kind of device failure, such as a disconnect.
    pub fn write(&mut self, data: &[u8]) -> SerialError {
        let res = self.raw_port.write_all(data);

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
        let res = self.raw_port.write_all(data.as_bytes());

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
        let read_num = self.raw_port.read(read_buff);

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
        let mut rust_buff = String::new();
        let cloned_port = self.raw_port.try_clone();

        if let Err(_) = cloned_port {
            return ReadResult {
                error: SerialError::PortIOErr,
                bytes_read: 0
            }
        }

        //We need a bufreader to use readline, as there is no EOF from a serial port.
        let mut reader = BufReader::new(cloned_port.unwrap());
        let read_num = reader.read_line(&mut rust_buff);

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

    //TODO add getters and setters for timout, bit size, ect.
    //TODO add a async reader using another thread and BufRead around the ports reader
}

/// Attempts to open the serial device at path, using the specified baud rate.
/// Defaults to a timeout of 99999 seconds.
pub fn open_port(path: &str, baud: u32) -> Result<Box<Serial>> {
    Ok(Box::from(Serial::new(path, baud)?))
}
