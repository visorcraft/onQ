//! Test fixture plugin for onQ.
//!
//! Registers a single `sample:say-hello` command. The current host records
//! its handler pointer but does not invoke it. The host loads this cdylib via
//! `onq_core::plugin::load` (see
//! `crates/onq-core/tests/integration/plugin_load.rs`) to test the C ABI.
//!
//! `API` is set during `onq_plugin_init` and read by `say_hello_handler`.
//! Both sites use `unsafe { ... }` because the host hands us a raw `*const`
//! pointer and function-table pointer with no Rust-side ownership. The pointer
//! is process-lifetime (the host owns the backing `HostApi`), so the static
//! `static mut` here never aliases another writer.

use onq_plugin_sdk::{export_plugin, HostApi};
use std::ffi::c_void;
use std::os::raw::c_char;

static mut API: *const HostApi = std::ptr::null();

/// Example handler for the future command-invocation ABI.
///
/// Reads the global `API` pointer and calls the host's `log` callback. The
/// current host records this pointer but does not call it. The `ctx` parameter
/// is reserved for future per-invocation state.
extern "C" fn say_hello_handler(
    _id: *const c_char,
    _args: *const c_char,
    _ctx: *mut c_void,
) -> i32 {
    // SAFETY: `API` is written once in `onq_plugin_init` before any
    // command can fire, and the host-owned `HostApi` outlives the plugin.
    unsafe {
        if !API.is_null() {
            ((*API).log)(
                1,
                b"sample-plugin: hello!\0".as_ptr() as *const _,
            );
        }
    }
    0
}

export_plugin!("sample-plugin", "0.1.0", "[]", |api: *const HostApi| -> i32 {
    // SAFETY: `api` is non-null and process-lifetime per the C ABI contract.
    unsafe {
        API = api;
        ((*api).register_command)(
            b"sample:say-hello\0".as_ptr() as *const _,
            b"Say Hello (sample)\0".as_ptr() as *const _,
            say_hello_handler as *const _,
        );
    }
    0
});
