use common::some_or_error;
use db::{get_dylib_path_by_name, insert_module, is_module_exists, CORE_DB_PATH};
use ehandle::{
    lpm::LpmError,
    module::{ModuleError, ModuleErrorKind},
    ErrorCommons,
};
use logger::{debug, info, success};
use min_sqlite3_sys::prelude::*;
use std::{ffi::CString, path::Path};

struct ModuleController(*mut std::os::raw::c_void);

const RTLD_NOW: std::os::raw::c_int = 0x2;

extern "C" {
    fn dlopen(
        filename: *const std::os::raw::c_char,
        flag: std::os::raw::c_int,
    ) -> *mut std::os::raw::c_void;

    fn dlsym(
        handle: *mut std::os::raw::c_void,
        symbol: *const std::os::raw::c_char,
    ) -> *mut std::os::raw::c_void;

    fn dlclose(handle: *mut std::os::raw::c_void) -> std::os::raw::c_int;
}

// We want to only pass the database path and command arguments so we don't need to
// worry about backwards compatibility(e.g when we add new fields to the configuration struct).
type ModuleEntrypointFn =
    extern "C" fn(*const std::os::raw::c_char, std::os::raw::c_uint, *const std::os::raw::c_void);

impl ModuleController {
    fn validate(dylib_path: &str) -> Result<(), LpmError<ModuleError>> {
        let mc = Self::load(dylib_path)?;

        let func_name = CString::new("lpm_entrypoint")?;

        #[allow(unsafe_code)]
        let func_ptr = unsafe { dlsym(mc.0, func_name.as_ptr()) };

        if func_ptr.is_null() {
            return Err(ModuleErrorKind::EntrypointFunctionNotFound.to_lpm_err());
        }

        Ok(())
    }

    fn load(dylib_path: &str) -> Result<Self, LpmError<ModuleError>> {
        let module = CString::new(dylib_path)?;

        #[allow(unsafe_code)]
        let lib_pointer = unsafe { dlopen(module.as_ptr(), RTLD_NOW) };

        if lib_pointer.is_null() {
            return Err(
                ModuleErrorKind::DynamicLibraryNotFound(dylib_path.to_owned()).to_lpm_err(),
            );
        }

        Ok(Self(lib_pointer))
    }

    fn run(&self, args: Vec<String>) -> Result<(), LpmError<ModuleError>> {
        let func_name = CString::new("lpm_entrypoint")?;

        #[allow(unsafe_code)]
        let func_ptr = unsafe { dlsym(self.0, func_name.as_ptr()) };

        if func_ptr.is_null() {
            return Err(ModuleErrorKind::EntrypointFunctionNotFound.to_lpm_err());
        }

        #[allow(unsafe_code)]
        let lpm_entrypoint: ModuleEntrypointFn = unsafe { std::mem::transmute(func_ptr) };

        let cstrings: Vec<CString> = args
            .iter()
            .map(|s| CString::new(s.as_str()).unwrap())
            .collect();
        let mut args_ptrs: Vec<*const std::os::raw::c_char> =
            cstrings.iter().map(|s| s.as_ptr()).collect();
        args_ptrs.push(std::ptr::null());

        let db_path = CString::new(CORE_DB_PATH)?;
        lpm_entrypoint(
            db_path.as_ptr(),
            (args_ptrs.len() - 1) as std::os::raw::c_uint,
            args_ptrs.as_ptr() as *const std::os::raw::c_void,
        );

        Ok(())
    }

    fn unload(self) {
        #[allow(unsafe_code)]
        unsafe {
            dlclose(self.0);
        }
    }
}

pub fn trigger_lpm_module(args: Vec<String>) -> Result<(), LpmError<ModuleError>> {
    let module_name = some_or_error!(
        args.get(2),
        "Provide the name of the module you wish to run."
    );

    let db = Database::open(Path::new(CORE_DB_PATH))?;

    let dylib_path = get_dylib_path_by_name(&db, module_name)?
        .ok_or_else(|| ModuleErrorKind::ModuleNotFound(module_name.to_owned()).to_lpm_err())?;

    db.close();

    let module_controller = ModuleController::load(&dylib_path)?;
    info!("Module '{}' loaded.", module_name);
    module_controller.run(args.clone())?;
    module_controller.unload();
    info!("Module '{}' finished running and unloaded.", module_name);

    Ok(())
}

pub fn add_module(name: &str, dylib_path: &str) -> Result<(), LpmError<ModuleError>> {
    // read absolute path of the dynamic library
    let dylib_path = std::fs::canonicalize(dylib_path)?;
    let dylib_path = dylib_path.to_string_lossy();

    let db = Database::open(Path::new(CORE_DB_PATH))?;

    if is_module_exists(&db, name)? {
        return Err(ModuleErrorKind::ModuleAlreadyExists(name.to_owned()).to_lpm_err());
    }

    // validate the module
    debug!("Validating {name} module..");
    ModuleController::validate(&dylib_path)?;

    info!("Adding {name} module to the database..");
    insert_module(&db, name, &dylib_path)?;
    db.close();
    success!("Operation successfully completed.");

    Ok(())
}

pub fn delete_modules(module_names: &[String]) -> Result<(), LpmError<ModuleError>> {
    if module_names.is_empty() {
        panic!("At least 1 module must be provided.");
    }

    let db = Database::open(Path::new(CORE_DB_PATH))?;

    for name in module_names {
        if !is_module_exists(&db, name)? {
            return Err(ModuleErrorKind::ModuleNotFound(name.to_owned()).to_lpm_err());
        }
    }

    info!("Deleting list of modules: {:?}", module_names);
    db::delete_modules(&db, module_names.to_vec())?;
    db.close();
    success!("Operation successfully completed.");

    Ok(())
}

pub fn print_modules() -> Result<(), LpmError<ModuleError>> {
    let db = Database::open(Path::new(CORE_DB_PATH))?;

    info!("Getting module list from the database..");
    let list = db::get_modules(&db)?;
    db.close();

    println!();

    if list.is_empty() {
        println!("No module has been found within the database.");
        return Ok(());
    }

    println!("Registered module list:");
    for item in list {
        println!("  {}: {}", item.0, item.1);
    }

    Ok(())
}
