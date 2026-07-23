//! Tauri commands for Settings → Backups (export / import / path display).
//!
//! Keeps archive I/O out of the monolithic `commands` module. Core logic
//! lives in [`onq_core::backup`]; this layer only:
//! - reads/writes [`AppState`] (including vault close before import)
//! - maps errors to strings for the IPC boundary
//! - returns UI-friendly DTOs

use std::path::PathBuf;

use onq_core::backup;
use serde::Serialize;
use tauri::State;

use crate::state::AppState;

/// Paths shown in the Backups panel (vault root + MongrelDB directory).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPathsDto {
    pub vault_path: String,
    pub database_path: String,
}

impl From<backup::BackupPaths> for BackupPathsDto {
    fn from(p: backup::BackupPaths) -> Self {
        Self {
            vault_path: p.vault_path.to_string_lossy().into_owned(),
            database_path: p.database_path.to_string_lossy().into_owned(),
        }
    }
}

/// Result of a successful import: vault is closed; UI should show unlock.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportBackupResult {
    pub path: String,
    /// True when the restored vault uses a master password (not keychain).
    pub needs_password: bool,
}

fn paths_dto(state: &AppState) -> Result<BackupPathsDto, String> {
    let vault = state.require_vault_path()?;
    Ok(backup::paths_for(vault).into())
}

/// Return the open vault path and its encrypted database directory.
#[tauri::command]
pub fn get_backup_paths(state: State<'_, AppState>) -> Result<BackupPathsDto, String> {
    paths_dto(&state)
}

/// True when `archive_path` is a password-sealed `.onqbak` (no password needed to probe).
#[tauri::command]
pub fn backup_is_sealed(archive_path: String) -> Result<bool, String> {
    backup::is_sealed_archive(PathBuf::from(archive_path).as_path()).map_err(|e| e.to_string())
}

/// Export the open vault to `dest_path` (`.onqbak`). Optional archive password.
#[tauri::command]
pub async fn export_vault_backup(
    dest_path: String,
    password: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let vault = state.require_vault_path()?;
    let dest = PathBuf::from(dest_path);
    let pw = password.filter(|s| !s.trim().is_empty());
    let db = state.db.lock().map_err(|e| e.to_string())?.clone();
    tokio::task::spawn_blocking(move || {
        backup::export_vault(&vault, &dest, pw.as_deref()).map_err(|e| e.to_string())?;
        if let Some(db) = db {
            let _ = db.set_app_setting("last_backup_at", &chrono::Utc::now().to_rfc3339());
        }
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Import a `.onqbak` over the **currently open** vault path, then close the session.
///
/// Destructive: replaces vault contents in place. Caller must confirm in UI.
#[tauri::command]
pub async fn import_vault_backup(
    archive_path: String,
    password: Option<String>,
    state: State<'_, AppState>,
) -> Result<ImportBackupResult, String> {
    let vault = state.require_vault_path()?;
    // Release MongrelDB file locks before rewriting the search-index tree.
    let closed = state.close_vault()?;
    let path = closed.unwrap_or(vault);
    let archive = PathBuf::from(archive_path);
    let pw = password.filter(|s| !s.trim().is_empty());
    let path_for_task = path.clone();

    tokio::task::spawn_blocking(move || {
        backup::import_vault(&archive, &path_for_task, pw.as_deref()).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    let needs_password = read_needs_password(&path);
    Ok(ImportBackupResult {
        path: path.to_string_lossy().into_owned(),
        needs_password,
    })
}

fn read_needs_password(vault_path: &std::path::Path) -> bool {
    let auth = vault_path.join(onq_core::backup::AUTH_MODE_REL);
    matches!(std::fs::read_to_string(auth), Ok(mode) if mode.trim() == "password")
}

#[cfg(test)]
mod tests {
    use super::*;
    use onq_core::backup::{self, layout};
    use std::fs;
    use tempfile::TempDir;

    fn seed(root: &std::path::Path) {
        fs::create_dir_all(layout::database_path(root)).unwrap();
        fs::write(layout::salt_path(root), [3u8; 32]).unwrap();
        fs::write(root.join(layout::AUTH_MODE_REL), b"password").unwrap();
        fs::write(root.join("p.md"), b"x").unwrap();
    }

    #[test]
    fn dto_from_paths() {
        let p = backup::paths_for("/tmp/vault");
        let dto: BackupPathsDto = p.into();
        assert!(dto.vault_path.contains("vault"));
        assert!(dto.database_path.contains("search-index"));
    }

    #[test]
    fn needs_password_from_auth_mode() {
        let dir = TempDir::new().unwrap();
        seed(dir.path());
        assert!(read_needs_password(dir.path()));
        fs::write(dir.path().join(layout::AUTH_MODE_REL), b"keychain").unwrap();
        assert!(!read_needs_password(dir.path()));
    }
}
