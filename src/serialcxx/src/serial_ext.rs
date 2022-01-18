use std::ffi::c_void;
use std::io::{Bytes, Chain, IoSliceMut, Read, Take};
use std::sync::{Arc, Mutex};
use serialport::SerialPort;

pub struct SerialPortReader {
    pub(crate) inner: Arc<Mutex<Box<dyn SerialPort>>>, //TODO change to parking lot mutex to make this double mutex nonsense fast
}

impl Read for SerialPortReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        return self.inner.lock().unwrap().read(buf);
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
        todo!()
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        todo!()
    }

    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        todo!()
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        todo!()
    }

    fn by_ref(&mut self) -> &mut Self
        where
            Self: Sized,
    {
        todo!()
    }

    fn bytes(self) -> Bytes<Self>
        where
            Self: Sized,
    {
        todo!()
    }

    fn chain<R: Read>(self, next: R) -> Chain<Self, R>
        where
            Self: Sized,
    {
        todo!()
    }

    fn take(self, limit: u64) -> Take<Self>
        where
            Self: Sized,
    {
        todo!()
    }
} //TODO just delegate all of these

#[derive(Copy, Clone)]
pub struct CVoidSend(pub *mut c_void);

unsafe impl Send for CVoidSend {}