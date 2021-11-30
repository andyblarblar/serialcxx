use crate::ffi::{CharSize, FlowControl, Parity, ReadResult, SerialError};
use cxx::CxxString;
use serialport::{DataBits, Result, SerialPort, StopBits};
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::pin::Pin;
use std::time::Duration;

pub struct Serial {
    raw_port: Box<dyn SerialPort>,
}

impl Serial {
    pub fn new(path: &str, baud: u32) -> Result<Serial> {
        let raw_port = serialport::new(path, baud)
            .timeout(Duration::from_secs(99999))
            .open()?;

        Ok(Serial { raw_port })
    }

    /// Sets the timeout for this port.
    ///
    /// Returns true if the operation succeeded.
    ///
    /// Note that settings changes will not propagate between the serial port and any open readers
    /// or other clones. Consider configuring the port before cloning.
    pub fn set_timeout(&mut self, sec: f32) -> bool {
        self.raw_port
            .set_timeout(Duration::from_secs_f32(sec))
            .is_ok()
    }

    /// Sets the character size of this port.
    ///
    /// Returns true if the operation succeeded.
    ///
    /// Note that settings changes will not propagate between the serial port and any open readers
    /// or other clones. Consider configuring the port before cloning.
    pub fn set_data_size(&mut self, bits: CharSize) -> bool {
        self.raw_port
            .set_data_bits(match bits {
                CharSize::Five => DataBits::Five,
                CharSize::Six => DataBits::Six,
                CharSize::Seven => DataBits::Seven,
                CharSize::Eight => DataBits::Eight,
                _ => DataBits::Eight
            })
            .is_ok()
    }

    /// Sets the baud rate of the port.
    ///
    /// Returns true if the operation succeeded.
    ///
    /// Note that settings changes will not propagate between the serial port and any open readers
    /// or other clones. Consider configuring the port before cloning.
    pub fn set_baud_rate(&mut self, baud: u32) -> bool {
        self.raw_port.set_baud_rate(baud).is_ok()
    }

    /// Sets the number of stop bits.
    /// True for two stop bits, false for one.
    ///
    /// Returns true if the operation succeeded.
    ///
    /// Note that settings changes will not propagate between the serial port and any open readers
    /// or other clones. Consider configuring the port before cloning.
    pub fn set_stop_bits(&mut self, two_bits: bool) -> bool {
        self.raw_port
            .set_stop_bits(if two_bits {
                StopBits::Two
            } else {
                StopBits::One
            })
            .is_ok()
    }

    /// Sets the parity checking mode.
    ///
    /// Returns true if the operation succeeded.
    ///
    /// Note that settings changes will not propagate between the serial port and any open readers
    /// or other clones. Consider configuring the port before cloning.
    pub fn set_parity(&mut self, mode: Parity) -> bool {
        self.raw_port
            .set_parity(match mode {
                Parity::Even => serialport::Parity::Even,
                Parity::Odd => serialport::Parity::Odd,
                Parity::None => serialport::Parity::None,
                _ => serialport::Parity::None
            })
            .is_ok()
    }

    /// Sets the flow control mode.
    ///
    /// Returns true if the operation succeeded.
    ///
    /// Note that settings changes will not propagate between the serial port and any open readers
    /// or other clones. Consider configuring the port before cloning.
    pub fn set_flow_control(&mut self, mode: FlowControl) -> bool {
        self.raw_port
            .set_flow_control(match mode {
                FlowControl::Hardware => serialport::FlowControl::Hardware,
                FlowControl::Software => serialport::FlowControl::Software,
                FlowControl::None => serialport::FlowControl::None,
                _ => serialport::FlowControl::None
            })
            .is_ok()
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
                bytes_read: 0,
            };
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

    //TODO add a async reader using another thread and BufRead around the ports reader
}

/// Attempts to open the serial device at path, using the specified baud rate.
/// Defaults to a timeout of 99999 seconds.
pub fn open_port(path: &str, baud: u32) -> Result<Box<Serial>> {
    Ok(Box::from(Serial::new(path, baud)?))
}
