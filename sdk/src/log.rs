use logger::{build_log, log_to_stderr, log_to_stdout, OutputMode};
use std::ffi::CStr;

macro_rules! create_stdout_log_fn {
    ($fn_name: ident, $log_mode: expr) => {
        #[no_mangle]
        extern "C" fn $fn_name(msg: *const std::os::raw::c_char) {
            let msg = unsafe { CStr::from_ptr(msg).to_bytes() };
            let log = build_log($log_mode, &String::from_utf8_lossy(msg));
            log_to_stdout(log.as_bytes());
        }
    };
}

create_stdout_log_fn!(success_log, OutputMode::SUCCESS);
create_stdout_log_fn!(info_log, OutputMode::INFO);
create_stdout_log_fn!(warning_log, OutputMode::WARNING);

#[no_mangle]
extern "C" fn error_log(msg: *const std::os::raw::c_char) {
    let msg = unsafe { CStr::from_ptr(msg).to_bytes() };
    let log = build_log(OutputMode::ERROR, &String::from_utf8_lossy(msg));
    log_to_stderr(log.as_bytes());
}

#[no_mangle]
#[cfg(debug_assertions)]
extern "C" fn debug_log(msg: *const std::os::raw::c_char) {
    let msg = unsafe { CStr::from_ptr(msg).to_bytes() };
    let log = build_log(OutputMode::DEBUG, &String::from_utf8_lossy(msg));
    log_to_stdout(log.as_bytes());
}

#[no_mangle]
#[cfg(not(debug_assertions))]
extern "C" fn debug_log(_msg: *const std::os::raw::c_char) {}
