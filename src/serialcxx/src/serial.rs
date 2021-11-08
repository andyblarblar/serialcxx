use serialport::Result;
use serialport::SerialPort;

pub struct Serial {
    raw_port: Box<dyn SerialPort>,
}

impl Serial {
    pub fn new(path: &str, baud: u32) -> Result<Serial> {
        Ok(Serial {
            raw_port: serialport::new(path, baud).open()?,
        })
    }

    pub fn callback(&self, fnc: extern "C" fn(input: u32) -> u32) {
        fnc(5);
    }
}

pub fn open_port(path: &str, baud: u32) -> Result<Box<Serial>> {
    Ok(Box::from(Serial::new(path, baud)?))
}
