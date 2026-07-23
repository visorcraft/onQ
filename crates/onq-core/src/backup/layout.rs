//! On-disk vault layout constants and recognition helpers.
//!
//! Future backup kinds (index-only, selective folder export) can share these
//! markers without re-deriving path conventions from Tauri commands.

use std::path::{Path, PathBuf};

/// Relative path of the encrypted MongrelDB directory inside a vault.
pub const SEARCH_INDEX_REL: &str = ".onq/search-index";
/// Relative path of the per-vault KEK salt inside a vault.
pub const SALT_REL: &str = ".onq/salt";
/// Relative path of the auth-mode marker (`password` | `keychain`).
pub const AUTH_MODE_REL: &str = ".onq/auth-mode";
/// Directory name under the vault root that holds MongrelDB + salt + state.
pub const ONQ_DIR: &str = ".onq";
/// MongrelDB directory name under [ONQ_DIR].
pub const SEARCH_INDEX_DIR: &str = "search-index";

/// Prefix used for temporary snapshots left during a failed import.
pub const PRE_IMPORT_PREFIX: &str = ".onq-pre-import-";

/// Absolute path of the encrypted search-index DB for `vault_path`.
pub fn database_path(vault_path: &Path) -> PathBuf {
    vault_path.join(ONQ_DIR).join(SEARCH_INDEX_DIR)
}

/// Absolute path of the per-vault salt file.
pub fn salt_path(vault_path: &Path) -> PathBuf {
    vault_path.join(SALT_REL)
}

/// True when `root` looks like an onQ vault (salt file + search-index dir).
pub fn is_vault_root(root: &Path) -> bool {
    salt_path(root).is_file() && database_path(root).is_dir()
}

/// Paths the Settings Backups panel (and future tools) surface to the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupPaths {
    pub vault_path: PathBuf,
    pub database_path: PathBuf,
}

impl BackupPaths {
    pub fn for_vault(vault_path: impl Into<PathBuf>) -> Self {
        let vault_path = vault_path.into();
        let database_path = database_path(&vault_path);
        Self {
            vault_path,
            database_path,
        }
    }
}

/// After unpacking an archive, locate the vault root.
///
/// Accepts either:
/// - archive contents at the root (`.onq/...`, `*.md`), or
/// - a single top-level directory that itself is a vault (folder-wrapped export).
pub fn resolve_vault_root(extracted: &Path) -> std::io::Result<PathBuf> {
    if is_vault_root(extracted) {
        return Ok(extracted.to_path_buf());
    }
    let mut dirs = Vec::new();
    for entry in std::fs::read_dir(extracted)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            dirs.push(entry.path());
        }
    }
    if dirs.len() == 1 && is_vault_root(&dirs[0]) {
        return Ok(dirs[0].clone());
    }
    Ok(extracted.to_path_buf())
}

/// Whether a directory entry name should be skipped when packing a vault
/// (import recovery leftovers, etc.).
pub fn should_skip_pack_entry(name: &str) -> bool {
    name.starts_with(PRE_IMPORT_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn seed(root: &Path) {
        fs::create_dir_all(database_path(root)).unwrap();
        fs::write(salt_path(root), [1u8; 32]).unwrap();
    }

    #[test]
    fn recognizes_seeded_vault() {
        let dir = TempDir::new().unwrap();
        assert!(!is_vault_root(dir.path()));
        seed(dir.path());
        assert!(is_vault_root(dir.path()));
    }

    #[test]
    fn resolve_unwraps_single_child_vault() {
        let outer = TempDir::new().unwrap();
        let inner = outer.path().join("MyVault");
        fs::create_dir_all(&inner).unwrap();
        seed(&inner);
        let resolved = resolve_vault_root(outer.path()).unwrap();
        assert_eq!(resolved, inner);
    }

    #[test]
    fn backup_paths_join_consistently() {
        let p = BackupPaths::for_vault("/tmp/v");
        assert_eq!(p.vault_path, PathBuf::from("/tmp/v"));
        assert_eq!(p.database_path, PathBuf::from("/tmp/v/.onq/search-index"));
    }
}
