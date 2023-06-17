use common::some_or_error;
use db::{get_dylib_path_by_name, insert_module, is_module_exists, CORE_DB_PATH};
use ehandle::{
    lpm::LpmError,
    module::{ModuleError, ModuleErrorKind},
    ErrorCommons,
};
use logger::{debug, info};
use min_sqlite3_sys::prelude::*;
use std::ffi::CString;

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
}

impl Drop for ModuleController {
    fn drop(&mut self) {
        #[allow(unsafe_code)]
        unsafe {
            dlclose(self.0);
        }
    }
}

pub fn trigger_lpm_module(
    core_db: &Database,
    args: Vec<String>,
) -> Result<(), LpmError<ModuleError>> {
    let module_name = some_or_error!(
        args.get(2),
        "Provide the name of the module you wish to run."
    );

    let dylib_path = get_dylib_path_by_name(core_db, module_name)?
        .ok_or_else(|| ModuleErrorKind::ModuleNotFound(module_name.to_owned()).to_lpm_err())?;

    info!("Module '{}' loaded.", module_name);
    let module_controller = ModuleController::load(&dylib_path)?;

    module_controller.run(args.clone())?;
    info!("Module '{}' finished running.", module_name);

    Ok(())
}

pub fn add_module(
    core_db: &Database,
    name: &str,
    dylib_path: &str,
) -> Result<(), LpmError<ModuleError>> {
    // read absolute path of the dynamic library
    let dylib_path = std::fs::canonicalize(dylib_path)?;
    let dylib_path = dylib_path.to_string_lossy();

    if is_module_exists(core_db, name)? {
        return Err(ModuleErrorKind::ModuleAlreadyExists(name.to_owned()).to_lpm_err());
    }

    // validate the module
    debug!("Validating {name} module..");
    ModuleController::validate(&dylib_path)?;

    info!("Adding {name} module to the database..");
    insert_module(core_db, name, &dylib_path)?;

    Ok(())
}

pub fn delete_modules(
    core_db: &Database,
    module_names: &[String],
) -> Result<(), LpmError<ModuleError>> {
    if module_names.is_empty() {
        panic!("At least 1 module must be provided.");
    }

    for name in module_names {
        if !is_module_exists(core_db, name)? {
            return Err(ModuleErrorKind::ModuleNotFound(name.to_owned()).to_lpm_err());
        }
    }

    info!("Deleting list of modules: {:?}", module_names);
    db::delete_modules(core_db, module_names.to_vec())?;

    Ok(())
}

pub fn print_modules(core_db: &Database) -> Result<(), LpmError<ModuleError>> {
    info!("Getting module list from the database..");
    let list = db::get_modules(core_db)?;

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
