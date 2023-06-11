use ehandle::ResultCode;
use std::ffi::CStr;

#[no_mangle]
extern "C" fn install_lod_file(pkg_path: *const std::os::raw::c_char) -> ResultCode {
    let pkg_path = unsafe {
        match CStr::from_ptr(pkg_path).to_str() {
            Ok(val) => val,
            Err(err) => {
                logger::error!("{}", err);
                return ResultCode::Str_Utf8Error;
            }
        }
    };

    let core_db = match core::open_core_db_connection() {
        Ok(t) => t,
        Err(err) => {
            logger::error!("{:?}", err);
            return err.result_code;
        }
    };

    if let Err(err) = core::install_from_lod_file(&core_db, pkg_path) {
        logger::error!("{:?}", err);
        return err.result_code;
    }

    ResultCode::Ok
}

#[no_mangle]
extern "C" fn update_from_lod_file(
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

    let core_db = match core::open_core_db_connection() {
        Ok(t) => t,
        Err(err) => {
            logger::error!("{:?}", err);
            return err.result_code;
        }
    };

    if let Err(err) = core::update_from_lod_file(&core_db, pkg_name, pkg_path) {
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

    let core_db = match core::open_core_db_connection() {
        Ok(t) => t,
        Err(err) => {
            logger::error!("{:?}", err);
            return err.result_code;
        }
    };

    if let Err(err) = core::delete_lod(&core_db, pkg_name) {
        logger::error!("{:?}", err);
        return err.result_code;
    }

    ResultCode::Ok
}
