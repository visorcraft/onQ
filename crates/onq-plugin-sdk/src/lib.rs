//! Helpers for plugin authors. Re-exports the ABI surface so plugins
//! don't need to redefine extern "C" signatures.

use std::os::raw::{c_char, c_void};

pub const ABI_VERSION: u32 = 1;

#[repr(C)]
pub struct HostApi {
    pub version: u32,
    pub db_query: unsafe extern "C" fn(sql: *const c_char, out: *mut c_char, out_len: usize) -> i32,
    pub db_execute: unsafe extern "C" fn(sql: *const c_char) -> i32,
    pub vault_read:
        unsafe extern "C" fn(path: *const c_char, out: *mut c_char, out_len: usize) -> i32,
    pub vault_write: unsafe extern "C" fn(path: *const c_char, data: *const c_char) -> i32,
    pub keychain_get:
        unsafe extern "C" fn(key: *const c_char, out: *mut c_char, out_len: usize) -> i32,
    pub keychain_set: unsafe extern "C" fn(key: *const c_char, value: *const c_char) -> i32,
    pub embedding_embed:
        unsafe extern "C" fn(text: *const c_char, out: *mut f32, dim: usize) -> i32,
    pub register_command:
        unsafe extern "C" fn(id: *const c_char, name: *const c_char, handler: *const c_void) -> i32,
    pub log: unsafe extern "C" fn(level: i32, message: *const c_char),
    pub config_get:
        unsafe extern "C" fn(key: *const c_char, out: *mut c_char, out_len: usize) -> i32,
    pub config_set: unsafe extern "C" fn(key: *const c_char, value: *const c_char) -> i32,
}

#[macro_export]
macro_rules! export_plugin {
    ($name:expr, $version:expr, $capabilities:expr, $init:expr) => {
        #[no_mangle]
        pub extern "C" fn onq_plugin_abi_version() -> u32 {
            1
        }
        #[no_mangle]
        pub extern "C" fn onq_plugin_name() -> *const std::os::raw::c_char {
            concat!($name, "\0").as_ptr() as *const _
        }
        #[no_mangle]
        pub extern "C" fn onq_plugin_version() -> *const std::os::raw::c_char {
            concat!($version, "\0").as_ptr() as *const _
        }
        #[no_mangle]
        pub extern "C" fn onq_plugin_capabilities() -> *const std::os::raw::c_char {
            concat!($capabilities, "\0").as_ptr() as *const _
        }
        #[no_mangle]
        pub extern "C" fn onq_plugin_init(api: *const $crate::HostApi) -> i32 {
            $init(api)
        }
    };
}
