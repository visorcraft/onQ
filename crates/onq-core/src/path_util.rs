//! Lexical path helpers shared by archive extractors (backups, plugins).
//!
//! These intentionally do **not** touch the filesystem: they collapse `.` /
//! `..` so archive entries can be validated before unpack.

use std::path::{Component, Path, PathBuf};

use crate::error::{CoreError, CoreResult};

/// Collapse `.` / `..` components without resolving symlinks or requiring
/// the path to exist.
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Join `entry_path` under `base` and reject any result that escapes `base`
/// after lexical normalization. Used to block `../` path-traversal in
/// untrusted archives.
pub fn safe_join(base: &Path, entry_path: &Path) -> CoreResult<PathBuf> {
    let joined = base.join(entry_path);
    let normalized = normalize_path(&joined);
    if !normalized.starts_with(base) {
        return Err(CoreError::Other(format!(
            "archive entry escapes staging dir: {}",
            entry_path.display()
        )));
    }
    Ok(normalized)
}

/// Recursively copy a file or directory tree from `from` to `to`.
pub fn copy_path(from: &Path, to: &Path) -> CoreResult<()> {
    let meta = std::fs::symlink_metadata(from)?;
    if meta.is_dir() {
        std::fs::create_dir_all(to)?;
        for entry in std::fs::read_dir(from)? {
            let entry = entry?;
            copy_path(&entry.path(), &to.join(entry.file_name()))?;
        }
    } else if meta.is_file() {
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(from, to)?;
    }
    // Symlinks / special files: skip (never materialize from archives).
    Ok(())
}

/// Remove a file or directory tree.
pub fn remove_path(path: &Path) -> CoreResult<()> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Move `from` to `to`, falling back to copy+delete across devices.
pub fn move_path(from: &Path, to: &Path) -> CoreResult<()> {
    if let Err(e) = std::fs::rename(from, to) {
        copy_path(from, to).map_err(|copy_err| {
            CoreError::Other(format!(
                "rename {} → {} failed ({e}); copy fallback: {copy_err}",
                from.display(),
                to.display()
            ))
        })?;
        remove_path(from)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn safe_join_blocks_parent_escape() {
        let base = Path::new("/tmp/stage");
        let err = safe_join(base, Path::new("../outside.txt")).unwrap_err();
        assert!(err.to_string().contains("escapes"));
    }

    #[test]
    fn safe_join_allows_nested() {
        let base = PathBuf::from("/tmp/stage");
        let joined = safe_join(&base, Path::new("a/b.txt")).unwrap();
        assert_eq!(joined, PathBuf::from("/tmp/stage/a/b.txt"));
    }

    #[test]
    fn move_path_roundtrip() {
        let dir = TempDir::new().unwrap();
        let from = dir.path().join("a.txt");
        let to = dir.path().join("b.txt");
        std::fs::write(&from, b"hi").unwrap();
        move_path(&from, &to).unwrap();
        assert!(!from.exists());
        assert_eq!(std::fs::read(&to).unwrap(), b"hi");
    }
}
