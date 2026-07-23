//! Full-vault backup archives (`.onqbak`).
//!
//! # Layers
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`layout`] | Vault path conventions + recognition |
//! | [`format`] | Versioned container (magic, plain/sealed) |
//! | [`payload`] | gzipped-tar of the vault tree |
//! | [`replace`] | In-place content swap with rollback |
//!
//! Public entry points [`export_vault`] / [`import_vault`] compose those
//! layers. Future features (index-only export, remote restore, streaming)
//! should extend a single layer rather than forking the façade.

pub mod format;
pub mod layout;
pub mod payload;
pub mod replace;

use std::path::Path;

use self::format::SealMode;
use self::layout::{is_vault_root, resolve_vault_root};
use crate::error::{CoreError, CoreResult};

pub use self::format::{is_sealed_archive, SealMode as BackupSealMode};
pub use self::layout::{
    database_path, salt_path, BackupPaths, AUTH_MODE_REL, SALT_REL, SEARCH_INDEX_REL,
};

/// Re-export for callers that need both paths for UI display.
pub type Paths = BackupPaths;

/// Pack the vault at `vault_path` into a `.onqbak` file at `dest`.
///
/// `password`: optional archive seal (independent of vault encryption).
/// Blank / `None` → plain container.
pub fn export_vault(vault_path: &Path, dest: &Path, password: Option<&str>) -> CoreResult<()> {
    if !vault_path.is_dir() {
        return Err(CoreError::Other(format!(
            "vault path is not a directory: {}",
            vault_path.display()
        )));
    }
    if !is_vault_root(vault_path) {
        return Err(CoreError::Other(format!(
            "not a valid onQ vault (missing {SALT_REL} or {SEARCH_INDEX_REL}): {}",
            vault_path.display()
        )));
    }

    let tree = payload::pack_tree(vault_path)?;
    let seal = SealMode::from_optional_password(password);
    format::write_container(dest, &tree, seal)
}

/// Replace the vault at `vault_path` with the contents of a `.onqbak` archive.
///
/// **Caller must close any open MongrelDB / file handles on this path first.**
/// On failure, previous vault contents are restored when possible.
pub fn import_vault(archive: &Path, vault_path: &Path, password: Option<&str>) -> CoreResult<()> {
    let tree = format::read_container(archive, password)?;
    let staging = tempfile::tempdir()?;
    payload::unpack_tree(&tree, staging.path())?;

    let staged_root = resolve_vault_root(staging.path())
        .map_err(|e| CoreError::Other(format!("resolve vault root: {e}")))?;
    if !is_vault_root(&staged_root) {
        return Err(CoreError::Other(
            "backup is not a valid onQ vault (missing .onq/salt or .onq/search-index)".into(),
        ));
    }

    replace::replace_with_staged(vault_path, &staged_root)
}

/// Convenience: paths to show in Settings → Backups.
pub fn paths_for(vault_path: impl Into<std::path::PathBuf>) -> BackupPaths {
    BackupPaths::for_vault(vault_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn seed_vault(root: &Path) {
        fs::create_dir_all(database_path(root)).unwrap();
        fs::write(salt_path(root), [9u8; 32]).unwrap();
        fs::write(root.join(".onq/auth-mode"), b"password").unwrap();
        fs::write(root.join("hello.md"), b"---\ntitle: Hi\n---\nbody\n").unwrap();
        fs::create_dir_all(root.join(".onq/history")).unwrap();
        fs::write(root.join(".onq/history/note.txt"), b"snap").unwrap();
    }

    #[test]
    fn export_import_plain_roundtrip() {
        let vault = TempDir::new().unwrap();
        seed_vault(vault.path());
        let archive_dir = TempDir::new().unwrap();
        let archive = archive_dir.path().join("plain.onqbak");

        export_vault(vault.path(), &archive, None).unwrap();
        assert!(archive.is_file());
        assert!(!is_sealed_archive(&archive).unwrap());

        let restored = TempDir::new().unwrap();
        fs::write(restored.path().join("stale.txt"), b"gone").unwrap();
        import_vault(&archive, restored.path(), None).unwrap();

        assert!(is_vault_root(restored.path()));
        assert_eq!(
            fs::read(restored.path().join("hello.md")).unwrap(),
            b"---\ntitle: Hi\n---\nbody\n"
        );
        assert!(!restored.path().join("stale.txt").exists());
    }

    #[test]
    fn export_import_password_roundtrip() {
        let vault = TempDir::new().unwrap();
        seed_vault(vault.path());
        let archive_dir = TempDir::new().unwrap();
        let archive = archive_dir.path().join("secret.onqbak");

        export_vault(vault.path(), &archive, Some("s3cret")).unwrap();
        assert!(is_sealed_archive(&archive).unwrap());

        let restored = TempDir::new().unwrap();
        let err = import_vault(&archive, restored.path(), None).unwrap_err();
        assert!(err.to_string().contains("password"), "got {err}");

        assert!(import_vault(&archive, restored.path(), Some("wrong")).is_err());
        import_vault(&archive, restored.path(), Some("s3cret")).unwrap();
        assert!(is_vault_root(restored.path()));
        assert_eq!(
            fs::read_to_string(restored.path().join(".onq/history/note.txt")).unwrap(),
            "snap"
        );
    }

    #[test]
    fn rejects_non_vault_export() {
        let dir = TempDir::new().unwrap();
        let archive = dir.path().join("x.onqbak");
        let err = export_vault(dir.path(), &archive, None).unwrap_err();
        assert!(err.to_string().contains("not a valid"));
    }

    #[test]
    fn rejects_garbage_magic() {
        let archive_dir = TempDir::new().unwrap();
        let archive = archive_dir.path().join("nope.onqbak");
        fs::write(&archive, b"not-a-backup").unwrap();
        let dest = TempDir::new().unwrap();
        let err = import_vault(&archive, dest.path(), None).unwrap_err();
        assert!(err.to_string().contains("not an onQ backup"), "got {err}");
    }
}
