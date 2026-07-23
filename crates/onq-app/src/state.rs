use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use onq_core::db::Db;
use onq_core::embed::Embedder;
use onq_core::vault::Vault;
use serde::Serialize;

use crate::auto_lock::AutoLockPolicy;

/// One palette-visible command registered by a loaded plugin (or host).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PluginCommand {
    pub id: String,
    pub name: String,
    pub plugin_id: String,
}

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
    /// Last user interaction timestamp for idle auto-lock evaluation.
    pub last_activity: Mutex<Instant>,
    /// Commands exposed by plugins for the palette (Wave 5.3).
    pub plugin_commands: Mutex<Vec<PluginCommand>>,
    /// Preferred embedder id: `"builtin"` or a plugin id with embedding cap.
    pub embedder_preference: Mutex<String>,
    /// When false, audit appends are no-ops (persisted in app config).
    pub audit_enabled: Mutex<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            vault: Mutex::new(None),
            vault_path: Mutex::new(None),
            db: Mutex::new(None),
            embedder: Mutex::new(None),
            auto_lock_policy: Mutex::new(AutoLockPolicy::LockOnQuit),
            last_activity: Mutex::new(Instant::now()),
            plugin_commands: Mutex::new(Vec::new()),
            embedder_preference: Mutex::new("builtin".into()),
            audit_enabled: Mutex::new(true),
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
    /// Used by backup import, lock-now, and switch-vault flows.
    pub fn close_vault(&self) -> Result<Option<PathBuf>, String> {
        let path = self.vault_path.lock().map_err(|e| e.to_string())?.take();
        *self.vault.lock().map_err(|e| e.to_string())? = None;
        // Drop Arc<Db> so MongrelDB releases its single-instance lock.
        *self.db.lock().map_err(|e| e.to_string())? = None;
        Ok(path)
    }

    /// Mark the current instant as the latest user activity (resets idle timer).
    pub fn touch_activity(&self) {
        if let Ok(mut guard) = self.last_activity.lock() {
            *guard = Instant::now();
        }
    }

    /// Register or replace a plugin palette command by `id`.
    pub fn register_plugin_command(&self, cmd: PluginCommand) -> Result<(), String> {
        let mut guard = self.plugin_commands.lock().map_err(|e| e.to_string())?;
        guard.retain(|c| c.id != cmd.id);
        guard.push(cmd);
        Ok(())
    }

    /// Drop all commands for a plugin id (on uninstall/disable).
    pub fn clear_plugin_commands(&self, plugin_id: &str) -> Result<(), String> {
        let mut guard = self.plugin_commands.lock().map_err(|e| e.to_string())?;
        guard.retain(|c| c.plugin_id != plugin_id);
        Ok(())
    }

    pub fn is_audit_enabled(&self) -> bool {
        self.audit_enabled.lock().map(|g| *g).unwrap_or(true)
    }
}
