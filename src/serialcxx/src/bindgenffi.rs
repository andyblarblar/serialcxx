use crate::SerialListenerBuilder;
use std::ffi::c_void;
use std::os::raw::c_char;

/// Adds the callback function to the serial listener.
/// This callback will be called for each /n/r or /n terminated line read from the serial port.
///
/// user_data will be passed into the user_data parameter in the callback on each invocation, allowing
/// the passing of arbitrary data into the callback. This can be a reference to a global, or a ref
/// to self to allow for member function invocation for example.
///
/// The remaining two arguments are the read string and it's size respectively.
///
/// The function will return false if the callback was not set due to null pointers being passed.
/// # Null policy
/// Listener must not be null, call must not be null, user_data may be null.
///
/// The string passed to the callback will never be null, but user_data will be if the passed user_data
/// was null.
#[no_mangle]
pub unsafe extern "C" fn add_read_callback(
    listener: *mut SerialListenerBuilder,
    user_data: *mut c_void, //Pointer to class instance to invoke with
    call: unsafe extern "C" fn(user_data: *mut c_void, string_read: *const c_char, str_size: usize), //Pass the above user data in with every call, as that void* is the this ptr. C cannot handle clases, so C++ must define a static wrapper to use with call here that will deref the passed userdata as a this* in order to call the member function. We dont need to do anything in rust other than pass in user data
) -> bool {
    if listener.is_null() {
        false
    } else {
        (*listener).callback = Some((user_data, call));
        true
    }
}
