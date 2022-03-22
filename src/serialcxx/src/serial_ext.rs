//! Extra types used in [serial].

use crate::Mutex;
use serialport::SerialPort;
use std::ffi::c_void;
use std::io::{IoSliceMut, Read};
use std::sync::Arc;

/// Internal Struct that wraps a Mutex protected serial port in a Read trait.
///
/// All Read operations first lock the serialport, then perform the Read method as defined by the
/// dyn SerialPort.
pub struct SerialPortReader {
    inner: Arc<Mutex<Box<dyn SerialPort>>>,
}

impl SerialPortReader {
    pub fn new(port: Arc<Mutex<Box<dyn SerialPort>>>) -> Self {
        SerialPortReader { inner: port }
    }
}

impl Read for SerialPortReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        return self.inner.lock().read(buf);
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
        return self.inner.lock().read_vectored(bufs);
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        return self.inner.lock().read_to_end(buf);
    }

    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        return self.inner.lock().read_to_string(buf);
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        return self.inner.lock().read_exact(buf);
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

/// c_void wrapper that impls send. We assume C++ has given us a thread safe pointer, so this tells
/// Rust that we believe such.
#[derive(Copy, Clone)]
pub struct CVoidSend(pub *mut c_void);

unsafe impl Send for CVoidSend {}
