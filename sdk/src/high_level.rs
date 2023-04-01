use ehandle::ResultCode;
use std::ffi::CStr;

#[no_mangle]
extern "C" fn install_lod(pkg_path: *const std::os::raw::c_char) -> ResultCode {
    let pkg_path = unsafe {
        match CStr::from_ptr(pkg_path).to_str() {
            Ok(val) => val,
            Err(err) => {
                logger::error!("{}", err);
                return ResultCode::Str_Utf8Error;
            }
        }
    };

    if let Err(err) = core::install_lod(pkg_path) {
        logger::error!("{:?}", err);
        return err.result_code;
    }

    ResultCode::Ok
}

#[no_mangle]
extern "C" fn update_lod(
    pkg_name: *const std::os::raw::c_char,
    pkg_path: *const std::os::raw::c_char,
) -> ResultCode {
    let pkg_name = unsafe {
        match CStr::from_ptr(pkg_name).to_str() {
            Ok(val) => val,
            Err(err) => {
                logger::error!("{}", err);
                return ResultCode::Str_Utf8Error;
            }
        }
    };

    let pkg_path = unsafe {
        match CStr::from_ptr(pkg_path).to_str() {
            Ok(val) => val,
            Err(err) => {
                logger::error!("{}", err);
                return ResultCode::Str_Utf8Error;
            }
        }
    };

    if let Err(err) = core::update_lod(pkg_name, pkg_path) {
        logger::error!("{:?}", err);
        return err.result_code;
    }

    ResultCode::Ok
}

#[no_mangle]
extern "C" fn delete_lod(pkg_name: *const std::os::raw::c_char) -> ResultCode {
    let pkg_name = unsafe {
        match CStr::from_ptr(pkg_name).to_str() {
            Ok(val) => val,
            Err(err) => {
                logger::error!("{}", err);
                return ResultCode::Str_Utf8Error;
            }
        }
    };

    if let Err(err) = core::delete_lod(pkg_name) {
        logger::error!("{:?}", err);
        return err.result_code;
    }

    ResultCode::Ok
}

#[no_mangle]
extern "C" fn add_pkg_kinds(
    kinds: *const *const std::os::raw::c_char,
    kinds_size: std::os::raw::c_uint,
) -> ResultCode {
    let kinds = unsafe { std::slice::from_raw_parts(kinds, kinds_size as usize) }
        .iter()
        .map(|&arg| unsafe { CStr::from_ptr(arg) })
        .map(|cstr| cstr.to_string_lossy().to_string())
        .collect::<Vec<String>>();

    if let Err(err) = core::add_pkg_kinds(&kinds) {
        logger::error!("{:?}", err);
        return err.result_code;
    }

    ResultCode::Ok
}

#[no_mangle]
extern "C" fn delete_pkg_kinds(
    kinds: *const *const std::os::raw::c_char,
    kinds_size: std::os::raw::c_uint,
) -> ResultCode {
    let kinds = unsafe { std::slice::from_raw_parts(kinds, kinds_size as usize) }
        .iter()
        .map(|&arg| unsafe { CStr::from_ptr(arg) })
        .map(|cstr| cstr.to_string_lossy().to_string())
        .collect::<Vec<String>>();

    if let Err(err) = core::delete_pkg_kinds(&kinds) {
        logger::error!("{:?}", err);
        return err.result_code;
    }

    ResultCode::Ok
}
