use std::ffi::c_void;
use std::os::raw::c_char;
use crate::serial::Serial;

#[no_mangle]
pub unsafe extern "C" fn add_read_callback(
    serial: *mut Serial,
    user_data: *mut c_void, //Pointer to class instance to invoke with
    call: extern "C" fn(user_data: *mut c_void, string_read: *const c_char, str_size: usize), //Pass the above user data in with every call, as that void* is the this ptr. C cannot handle clases, so C++ must define a static wrapper to use with call here that will deref the passed userdata as a this* in order to call the member function. We dont need to do anything in rust other than pass in user data
) {
}
