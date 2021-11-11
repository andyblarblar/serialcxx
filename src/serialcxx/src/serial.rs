use serialport::{Result, SerialPort};
use std::io::{ErrorKind, Write};
use crate::ffi::SerialError;

pub struct Serial {
    raw_port: Box<dyn SerialPort>,
}

impl Serial {
    pub fn new(path: &str, baud: u32) -> Result<Serial> {
        Ok(Serial {
            raw_port: serialport::new(path, baud).open()?,
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

    pub fn callback(&self, fnc: extern "C" fn(input: u32) -> u32) {
        fnc(5);
    }
}

pub fn open_port(path: &str, baud: u32) -> Result<Box<Serial>> {
    Ok(Box::from(Serial::new(path, baud)?))
}
