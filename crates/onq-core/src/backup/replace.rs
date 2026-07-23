//! In-place directory content replacement with rollback.
//!
//! Used by vault import: stage a new tree, move live contents aside, promote
//! the staged tree, and restore on failure. Kept free of archive format so
//! future restore paths (e.g. folder pick without `.onqbak`) can reuse it.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::layout::PRE_IMPORT_PREFIX;
use crate::error::CoreResult;
use crate::path_util;

/// Replace everything under `target` with the children of `staged_root`.
///
/// Steps:
/// 1. Move live entries into a sibling `.onq-pre-import-<ts>/` snapshot.
/// 2. Copy staged children into `target`.
/// 3. On success, delete the snapshot (and any older pre-import leftovers).
/// 4. On failure, restore the snapshot and return the error.
pub fn replace_with_staged(target: &Path, staged_root: &Path) -> CoreResult<()> {
    std::fs::create_dir_all(target)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup_name = format!("{PRE_IMPORT_PREFIX}{stamp}");
    let backup_dir = target.join(&backup_name);
    std::fs::create_dir_all(&backup_dir)?;

    let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(target)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == backup_name || name_str.starts_with(PRE_IMPORT_PREFIX) {
            continue;
        }
        let from = entry.path();
        let to = backup_dir.join(&name);
        if let Err(e) = path_util::move_path(&from, &to) {
            restore_moved(&moved);
            let _ = path_util::remove_path(&backup_dir);
            return Err(e);
        }
        moved.push((from, to));
    }

    for entry in std::fs::read_dir(staged_root)? {
        let entry = entry?;
        let dest = target.join(entry.file_name());
        if let Err(e) = path_util::copy_path(&entry.path(), &dest) {
            clear_target_except_backup(target, &backup_name);
            restore_moved(&moved);
            let _ = path_util::remove_path(&backup_dir);
            return Err(e);
        }
    }

    let _ = path_util::remove_path(&backup_dir);
    cleanup_pre_import(target);
    Ok(())
}

fn restore_moved(moved: &[(PathBuf, PathBuf)]) {
    for (orig, bak) in moved.iter().rev() {
        let _ = path_util::move_path(bak, orig);
    }
}

fn clear_target_except_backup(target: &Path, backup_name: &str) {
    let Ok(rd) = std::fs::read_dir(target) else {
        return;
    };
    for entry in rd.flatten() {
        if entry.file_name().to_string_lossy() == backup_name {
            continue;
        }
        let _ = path_util::remove_path(&entry.path());
    }
}

fn cleanup_pre_import(target: &Path) {
    let Ok(rd) = std::fs::read_dir(target) else {
        return;
    };
    for entry in rd.flatten() {
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with(PRE_IMPORT_PREFIX)
        {
            let _ = path_util::remove_path(&entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn replaces_and_removes_stale() {
        let target = TempDir::new().unwrap();
        fs::write(target.path().join("old.txt"), b"old").unwrap();
        let staged = TempDir::new().unwrap();
        fs::write(staged.path().join("new.txt"), b"new").unwrap();
        fs::create_dir_all(staged.path().join("sub")).unwrap();
        fs::write(staged.path().join("sub/a"), b"a").unwrap();

        replace_with_staged(target.path(), staged.path()).unwrap();

        assert!(!target.path().join("old.txt").exists());
        assert_eq!(fs::read(target.path().join("new.txt")).unwrap(), b"new");
        assert_eq!(fs::read(target.path().join("sub/a")).unwrap(), b"a");
        assert!(!target.path().read_dir().unwrap().any(|e| {
            e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(PRE_IMPORT_PREFIX)
        }));
    }
}
