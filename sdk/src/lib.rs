#![allow(unsafe_code)]

mod high_level;
mod log;

use std::ffi::CString;

use ehandle::ResultCode;

#[repr(C)]
struct CVersionStruct {
    pub readable_format: *const std::os::raw::c_char,
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub tag: *const std::os::raw::c_char,
}

#[no_mangle]
extern "C" fn get_lpm_version(vref: &mut CVersionStruct) -> ResultCode {
    let common::version::VersionStruct {
        readable_format,
        major,
        minor,
        patch,
        tag,
    } = common::lpm_version::get_lpm_version();

    let readable_format = match CString::new(readable_format) {
        Ok(val) => val,
        Err(e) => {
            logger::error!("{}", e);
            return ResultCode::CStr_NulError;
        }
    };

    let tag = match CString::new(tag.unwrap_or_default()) {
        Ok(val) => val,
        Err(e) => {
            logger::error!("{}", e);
            return ResultCode::CStr_NulError;
        }
    };

    vref.readable_format = readable_format.into_raw();
    vref.major = major;
    vref.minor = minor;
    vref.patch = patch;
    vref.tag = tag.into_raw();

    ResultCode::Ok
}
