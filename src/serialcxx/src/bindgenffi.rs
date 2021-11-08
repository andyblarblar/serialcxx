use crate::serial::Serial;

#[no_mangle]
pub unsafe extern "C" fn add_read_callback(serial: *mut Serial, call: extern "C" fn(input:u32) -> u32) {
    (*serial).callback(call);
}