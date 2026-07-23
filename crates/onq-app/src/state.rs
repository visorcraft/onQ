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
    pub embedder: Mutex<Option<Arc<Mutex<Embedder>>>>,
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

impl AppState {
    /// Absolute path of the currently open vault, if any.
    pub fn open_vault_path(&self) -> Result<Option<PathBuf>, String> {
        Ok(self.vault_path.lock().map_err(|e| e.to_string())?.clone())
    }

    /// Require an open vault path (errors when locked / never opened).
    pub fn require_vault_path(&self) -> Result<PathBuf, String> {
        self.open_vault_path()?
            .ok_or_else(|| "vault not unlocked".into())
    }

    /// Drop the open vault + DB handles so files can be replaced on disk.
    ///
    /// Returns the path that was open (if any). Embedder is left loaded —
    /// it is process-global model state, not vault-scoped.
    ///
    /// Used by backup import and (later) explicit lock / switch-vault flows.
    pub fn close_vault(&self) -> Result<Option<PathBuf>, String> {
        let path = self.vault_path.lock().map_err(|e| e.to_string())?.take();
        *self.vault.lock().map_err(|e| e.to_string())? = None;
        // Drop Arc<Db> so MongrelDB releases its single-instance lock.
        *self.db.lock().map_err(|e| e.to_string())? = None;
        Ok(path)
    }
}
