//! Test fixture plugin for onQ.
//!
//! Registers a single `sample:say-hello` command that logs a greeting when
//! invoked. The host loads this cdylib via `onq_core::plugin::load`
//! (see `crates/onq-core/tests/integration/plugin_load.rs`) to prove
//! the C ABI round-trips end-to-end.
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

/// Handler invoked when the user runs "Say Hello (sample)" from the palette.
/// Reads the global `API` pointer and calls back into the host's `log` fn.
/// The `ctx` parameter is reserved for future per-invocation state.
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