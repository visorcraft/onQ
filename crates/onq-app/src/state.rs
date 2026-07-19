use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use onq_core::db::Db;
use onq_core::embed::Embedder;
use onq_core::vault::Vault;

use crate::auto_lock::AutoLockPolicy;

pub struct AppState {
    pub vault: Mutex<Option<Vault>>,
    pub vault_path: Mutex<Option<PathBuf>>,
    /// Open encrypted search-index DB. Wrapped in `Arc` so the `search`
    /// Tauri command can clone it into `spawn_blocking` without owning
    /// the lock for the duration of the (sync, CPU-bound) DB calls.
    pub db: Mutex<Option<Arc<Db>>>,
    /// ONNX embedder. Optional until the model is downloaded; `search`
    /// degrades to keyword-only when this is `None`.
    pub embedder: Mutex<Option<Arc<Embedder>>>,
    /// Active auto-lock policy. Defaults to `LockOnQuit` so the user
    /// always gets the safe baseline; the frontend can downgrade to
    /// `Disabled` or upgrade to `IdleTimeout(...)` via the
    /// `set_auto_lock_policy` Tauri command.
    pub auto_lock_policy: Mutex<AutoLockPolicy>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            vault: Mutex::new(None),
            vault_path: Mutex::new(None),
            db: Mutex::new(None),
            embedder: Mutex::new(None),
            auto_lock_policy: Mutex::new(AutoLockPolicy::LockOnQuit),
        }
    }
}
