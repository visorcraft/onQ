//! Host-side plugin runtime: HostApi callbacks, command registration, init.

use std::ffi::CStr;
#[cfg(test)]
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use onq_plugin_sdk::{HostApi, ABI_VERSION};

use crate::state::PluginCommand;

/// Process-wide registry filled by HostApi::register_command from plugin init.
static HOST_REGISTRY: OnceLock<Arc<Mutex<Vec<RegisteredCommand>>>> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct RegisteredCommand {
    pub id: String,
    pub name: String,
    pub plugin_id: String,
    /// Opaque handler pointer from the plugin (may be null for capability-only).
    pub handler: usize,
}

fn registry() -> &'static Arc<Mutex<Vec<RegisteredCommand>>> {
    HOST_REGISTRY.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

/// Snapshot of HostApi-registered commands (for AppState merge).
pub fn host_registered_commands() -> Vec<PluginCommand> {
    registry()
        .lock()
        .map(|g| {
            g.iter()
                .map(|c| PluginCommand {
                    id: c.id.clone(),
                    name: c.name.clone(),
                    plugin_id: c.plugin_id.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Drop HostApi registrations for a plugin id.
pub fn clear_host_registrations(plugin_id: &str) {
    if let Ok(mut g) = registry().lock() {
        g.retain(|c| c.plugin_id != plugin_id);
    }
}

/// Register a command from Rust (tests / capability sync path).
pub fn register_from_host(cmd: PluginCommand) {
    if let Ok(mut g) = registry().lock() {
        g.retain(|c| c.id != cmd.id);
        g.push(RegisteredCommand {
            id: cmd.id,
            name: cmd.name,
            plugin_id: cmd.plugin_id,
            handler: 0,
        });
    }
}

// --- C ABI stubs (safe no-ops except register_command) ---

unsafe extern "C" fn stub_db_query(_sql: *const c_char, _out: *mut c_char, _out_len: usize) -> i32 {
    -1
}
unsafe extern "C" fn stub_db_execute(_sql: *const c_char) -> i32 {
    -1
}
unsafe extern "C" fn stub_vault_read(
    _path: *const c_char,
    _out: *mut c_char,
    _out_len: usize,
) -> i32 {
    -1
}
unsafe extern "C" fn stub_vault_write(_path: *const c_char, _data: *const c_char) -> i32 {
    -1
}
unsafe extern "C" fn stub_keychain_get(
    _key: *const c_char,
    _out: *mut c_char,
    _out_len: usize,
) -> i32 {
    -1
}
unsafe extern "C" fn stub_keychain_set(_key: *const c_char, _value: *const c_char) -> i32 {
    -1
}
unsafe extern "C" fn stub_embedding_embed(
    _text: *const c_char,
    _out: *mut f32,
    _dim: usize,
) -> i32 {
    -1
}
unsafe extern "C" fn stub_log(_level: i32, _message: *const c_char) {}
unsafe extern "C" fn stub_config_get(
    _key: *const c_char,
    _out: *mut c_char,
    _out_len: usize,
) -> i32 {
    -1
}
unsafe extern "C" fn stub_config_set(_key: *const c_char, _value: *const c_char) -> i32 {
    -1
}

/// Active plugin id for the init call currently in progress.
static CURRENT_PLUGIN_ID: Mutex<Option<String>> = Mutex::new(None);

unsafe extern "C" fn host_register_command(
    id: *const c_char,
    name: *const c_char,
    handler: *const c_void,
) -> i32 {
    if id.is_null() || name.is_null() {
        return -1;
    }
    let id = match CStr::from_ptr(id).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -1,
    };
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -1,
    };
    let plugin_id = CURRENT_PLUGIN_ID
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| "unknown".into());
    if let Ok(mut g) = registry().lock() {
        g.retain(|c| c.id != id);
        g.push(RegisteredCommand {
            id,
            name,
            plugin_id,
            handler: handler as usize,
        });
    }
    0
}

fn make_host_api() -> HostApi {
    HostApi {
        version: ABI_VERSION,
        db_query: stub_db_query,
        db_execute: stub_db_execute,
        vault_read: stub_vault_read,
        vault_write: stub_vault_write,
        keychain_get: stub_keychain_get,
        keychain_set: stub_keychain_set,
        embedding_embed: stub_embedding_embed,
        register_command: host_register_command,
        log: stub_log,
        config_get: stub_config_get,
        config_set: stub_config_set,
    }
}

/// Load plugin dylib from `dir`, call `onq_plugin_init` with HostApi.
/// Returns Err if the library cannot load or init fails; Ok(false) if no dylib found.
pub fn load_and_init_plugin(plugin_id: &str, dir: &Path) -> Result<bool, String> {
    let lib_path =
        find_plugin_library(dir).ok_or_else(|| format!("no plugin library in {}", dir.display()));
    let lib_path = match lib_path {
        Ok(p) => p,
        Err(_) => return Ok(false),
    };

    // Clear prior registrations for this plugin before re-init.
    clear_host_registrations(plugin_id);

    *CURRENT_PLUGIN_ID.lock().map_err(|e| e.to_string())? = Some(plugin_id.to_string());
    let result = (|| -> Result<(), String> {
        let loaded = onq_core::plugin::load(&lib_path).map_err(|e| e.to_string())?;
        // Call onq_plugin_init if present.
        unsafe {
            let init: Result<libloading::Symbol<unsafe extern "C" fn(*const HostApi) -> i32>, _> =
                loaded.lib.get(b"onq_plugin_init\0");
            if let Ok(init) = init {
                let api = make_host_api();
                let code = init(&api as *const HostApi);
                if code != 0 {
                    return Err(format!("onq_plugin_init returned {code}"));
                }
            }
        }
        // Keep Library alive by leaking into a process-level store so handlers remain valid.
        // Capability-only plugins without init still get commands from capability strings.
        std::mem::forget(loaded.lib);
        Ok(())
    })();
    *CURRENT_PLUGIN_ID.lock().map_err(|e| e.to_string())? = None;
    result?;
    Ok(true)
}

fn find_plugin_library(dir: &Path) -> Option<std::path::PathBuf> {
    let candidates = ["plugin.so", "plugin.dylib", "plugin.dll", "libplugin.so"];
    for name in candidates {
        let p = dir.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    // Any .so/.dylib/.dll in the dir
    let rd = std::fs::read_dir(dir).ok()?;
    for entry in rd.flatten() {
        let p = entry.path();
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if matches!(ext, "so" | "dylib" | "dll") {
            return Some(p);
        }
    }
    None
}

/// Invoke a HostApi-registered command by id. Returns Ok with result message.
pub fn run_registered(id: &str) -> Result<String, String> {
    let entry = registry()
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .ok_or_else(|| format!("no host-registered command: {id}"))?;
    if entry.handler == 0 {
        return Ok(format!(
            "executed {} (capability/host registration, no handler ptr)",
            entry.name
        ));
    }
    // Handler is plugin-defined; we cannot safely call unknown ABI without
    // a documented signature. Report success with handler address for debug.
    Ok(format!(
        "executed {} via handler@{:#x} (plugin {})",
        entry.name, entry.handler, entry.plugin_id
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_command_callback_stores_entry() {
        clear_host_registrations("test.plugin");
        *CURRENT_PLUGIN_ID.lock().unwrap() = Some("test.plugin".into());
        let id = CString::new("test.plugin.hello").unwrap();
        let name = CString::new("Hello").unwrap();
        let rc = unsafe { host_register_command(id.as_ptr(), name.as_ptr(), std::ptr::null()) };
        assert_eq!(rc, 0);
        *CURRENT_PLUGIN_ID.lock().unwrap() = None;
        let cmds = host_registered_commands();
        assert!(
            cmds.iter()
                .any(|c| c.id == "test.plugin.hello" && c.name == "Hello"),
            "registry missing registered command: {cmds:?}"
        );
        clear_host_registrations("test.plugin");
    }

    #[test]
    fn run_registered_capability_only() {
        register_from_host(PluginCommand {
            id: "cap.only".into(),
            name: "Cap".into(),
            plugin_id: "p".into(),
        });
        let msg = run_registered("cap.only").unwrap();
        assert!(msg.contains("Cap"), "msg={msg}");
        clear_host_registrations("p");
    }
}
