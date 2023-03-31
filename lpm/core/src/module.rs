use common::{
    config::{LpmConfig, CONFIG_PATH},
    ParserTasks,
};
use db::DB_PATH;
use ehandle::{
    lpm::LpmError,
    module::{ModuleError, ModuleErrorKind},
    ErrorCommons,
};
use logger::info;
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

// We want to only pass configuration and database path and command arguments so we don't
// need to worry about backwards compatibility(e.g when we add new fields to the configuration struct).
type ModuleEntrypointFn = extern "C" fn(
    *const std::os::raw::c_char,
    *const std::os::raw::c_char,
    std::os::raw::c_uint,
    *const std::os::raw::c_void,
);

impl ModuleController {
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

        let config_path = CString::new(CONFIG_PATH)?;
        let db_path = CString::new(DB_PATH)?;
        lpm_entrypoint(
            config_path.as_ptr(),
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
    let module_name = args.get(2).expect("Module name is missing.");
    let lpm_config = LpmConfig::deserialize(CONFIG_PATH);
    let module = lpm_config
        .modules
        .iter()
        .find(|p| p.name == *module_name)
        .unwrap_or_else(|| panic!("Module '{}' not found", module_name));

    let module_controller = ModuleController::load(&module.dylib_path)?;
    info!("Module '{}' loaded.", module_name);
    module_controller.run(args.clone())?;
    module_controller.unload();
    info!("Module '{}' finished running and unloaded.", module_name);

    Ok(())
}
