use std::ffi::CStr;

#[no_mangle]
extern "C" fn install_lod(pkg_path: *const std::os::raw::c_char) {
    let pkg_path = unsafe { CStr::from_ptr(pkg_path).to_str().unwrap() };
    core::install_lod(pkg_path).unwrap();
}

#[no_mangle]
extern "C" fn update_lod(
    pkg_name: *const std::os::raw::c_char,
    pkg_path: *const std::os::raw::c_char,
) {
    let pkg_name = unsafe { CStr::from_ptr(pkg_name).to_str().unwrap() };
    let pkg_path = unsafe { CStr::from_ptr(pkg_path).to_str().unwrap() };
    core::update_lod(pkg_name, pkg_path).unwrap();
}

#[no_mangle]
extern "C" fn delete_lod(pkg_name: *const std::os::raw::c_char) {
    let pkg_name = unsafe { CStr::from_ptr(pkg_name).to_str().unwrap() };
    core::delete_lod(pkg_name).unwrap();
}

#[no_mangle]
extern "C" fn add_pkg_kinds(
    kinds: *const *const std::os::raw::c_char,
    kinds_size: std::os::raw::c_uint,
) {
    let kinds = unsafe { std::slice::from_raw_parts(kinds, kinds_size as usize) }
        .iter()
        .map(|&arg| unsafe { CStr::from_ptr(arg) })
        .map(|cstr| cstr.to_string_lossy().to_string())
        .collect::<Vec<String>>();

    core::add_pkg_kinds(&kinds).unwrap();
}

#[no_mangle]
extern "C" fn delete_pkg_kinds(
    kinds: *const *const std::os::raw::c_char,
    kinds_size: std::os::raw::c_uint,
) {
    let kinds = unsafe { std::slice::from_raw_parts(kinds, kinds_size as usize) }
        .iter()
        .map(|&arg| unsafe { CStr::from_ptr(arg) })
        .map(|cstr| cstr.to_string_lossy().to_string())
        .collect::<Vec<String>>();

    core::delete_pkg_kinds(&kinds).unwrap();
}
