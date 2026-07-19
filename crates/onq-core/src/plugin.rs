use crate::error::{CoreError, CoreResult};
use libloading::Library;
use std::path::Path;

pub struct LoadedPlugin {
    pub lib: Library,
    pub name: String,
    pub version: String,
    pub capabilities: serde_json::Value,
}

impl std::fmt::Debug for LoadedPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedPlugin")
            .field("name", &self.name)
            .field("version", &self.version)
            .field("capabilities", &self.capabilities)
            .finish_non_exhaustive()
    }
}

pub fn load(path: &Path) -> CoreResult<LoadedPlugin> {
    unsafe {
        let lib = Library::new(path).map_err(|e| CoreError::Plugin(e.to_string()))?;
        let abi: libloading::Symbol<unsafe extern "C" fn() -> u32> = lib
            .get(b"onq_plugin_abi_version\0")
            .map_err(|e| CoreError::Plugin(e.to_string()))?;
        let v = abi();
        if v != onq_plugin_sdk::ABI_VERSION {
            return Err(CoreError::Plugin(format!(
                "abi mismatch: host={} plugin={}",
                onq_plugin_sdk::ABI_VERSION,
                v
            )));
        }
        let name = read_cstr(&lib, b"onq_plugin_name\0")?;
        let version = read_cstr(&lib, b"onq_plugin_version\0")?;
        let caps = read_cstr(&lib, b"onq_plugin_capabilities\0")?;
        let capabilities: serde_json::Value =
            serde_json::from_str(&caps).map_err(|e| CoreError::Plugin(e.to_string()))?;
        Ok(LoadedPlugin {
            lib,
            name,
            version,
            capabilities,
        })
    }
}

fn read_cstr(lib: &Library, sym: &[u8]) -> CoreResult<String> {
    unsafe {
        let f: libloading::Symbol<unsafe extern "C" fn() -> *const std::os::raw::c_char> =
            lib.get(sym).map_err(|e| CoreError::Plugin(e.to_string()))?;
        let ptr = f();
        if ptr.is_null() {
            return Err(CoreError::Plugin("null cstr".into()));
        }
        Ok(std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Tests will use the sample plugin from M6.4
    #[test]
    fn load_nonexistent_returns_plugin_error() {
        let p = std::path::Path::new("/nonexistent-onq-plugin-xyz.so");
        let err = load(p).unwrap_err();
        match err {
            CoreError::Plugin(_) => {}
            other => panic!("expected Plugin error, got {other:?}"),
        }
    }
}
