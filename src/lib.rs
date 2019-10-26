use std::os::raw::c_char;
use std::ffi::CStr;

extern "C" fn panic_handler(msg: *const c_char) {
    let msg = unsafe {
        CStr::from_ptr(msg)
    }.to_string_lossy();

    panic!("{}", msg);
}

extern "C" fn logger(log_level: u32, msg: *const c_char) {
    let msg = unsafe {
        CStr::from_ptr(msg)
    }.to_string_lossy();

    match log_level {
        akumuli_sys::aku_LogLevel_AKU_LOG_TRACE => log::trace!("{}", msg),
        akumuli_sys::aku_LogLevel_AKU_LOG_INFO => log::info!("{}", msg),
        akumuli_sys::aku_LogLevel_AKU_LOG_ERROR => log::error!("{}", msg),
        _ => log::debug!("{}", msg),
    }
}

pub fn initialize() {
    unsafe {
        akumuli_sys::aku_initialize(Some(panic_handler), Some(logger));
    }
}
