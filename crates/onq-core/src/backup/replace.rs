//! In-place directory content replacement with rollback.
//!
//! Used by vault import: stage a new tree, move live contents aside, promote
//! the staged tree, and restore on failure. Kept free of archive format so
//! future restore paths (e.g. folder pick without `.onqbak`) can reuse it.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::layout::PRE_IMPORT_PREFIX;
use crate::error::{CoreError, CoreResult};
use crate::path_util;

/// Replace everything under `target` with the children of `staged_root`.
///
/// Steps:
/// 1. Move live entries into a sibling `.onq-pre-import-<ts>/` snapshot.
/// 2. Copy staged children into `target`.
/// 3. On success, delete the snapshot (and any older pre-import leftovers).
/// 4. On failure, restore the snapshot and return the error. If the restore
///    is incomplete, the snapshot is kept and its path is reported in the
///    error message.
pub fn replace_with_staged(target: &Path, staged_root: &Path) -> CoreResult<()> {
    replace_with_staged_using(target, staged_root, path_util::copy_path)
}

fn replace_with_staged_using(
    target: &Path,
    staged_root: &Path,
    mut copy: impl FnMut(&Path, &Path) -> CoreResult<()>,
) -> CoreResult<()> {
    std::fs::create_dir_all(target)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup_name = format!("{PRE_IMPORT_PREFIX}{stamp}");
    let backup_dir = target.join(&backup_name);
    std::fs::create_dir_all(&backup_dir)?;

    let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut move_failed: Option<CoreError> = None;
    for entry in std::fs::read_dir(target)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                move_failed = Some(CoreError::from(e));
                break;
            }
        };
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == backup_name || name_str.starts_with(PRE_IMPORT_PREFIX) {
            continue;
        }
        let from = entry.path();
        let to = backup_dir.join(&name);
        if let Err(e) = path_util::move_path(&from, &to) {
            move_failed = Some(e);
            break;
        }
        moved.push((from, to));
    }
    if let Some(cause) = move_failed {
        return Err(rollback(target, &backup_dir, &moved, false, cause));
    }

    let staged_rd = match std::fs::read_dir(staged_root) {
        Ok(rd) => rd,
        Err(e) => {
            return Err(rollback(
                target,
                &backup_dir,
                &moved,
                true,
                CoreError::from(e),
            ));
        }
    };
    let mut copy_failed: Option<CoreError> = None;
    for entry in staged_rd {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                copy_failed = Some(CoreError::from(e));
                break;
            }
        };
        let dest = target.join(entry.file_name());
        if let Err(e) = copy(&entry.path(), &dest) {
            copy_failed = Some(e);
            break;
        }
    }
    if let Some(cause) = copy_failed {
        return Err(rollback(target, &backup_dir, &moved, true, cause));
    }

    let _ = path_util::remove_path(&backup_dir);
    cleanup_pre_import(target);
    Ok(())
}

fn rollback(
    target: &Path,
    backup_dir: &Path,
    moved: &[(PathBuf, PathBuf)],
    clear_copied: bool,
    cause: CoreError,
) -> CoreError {
    if clear_copied {
        clear_target_except_backup(target);
    }
    let mut failed = 0usize;
    for (orig, bak) in moved.iter().rev() {
        if path_util::move_path(bak, orig).is_err() {
            failed += 1;
        }
    }
    if failed == 0 {
        let _ = path_util::remove_path(backup_dir);
        return cause;
    }
    CoreError::Other(format!(
        "rollback failed: {failed} of {} moved entries could not be restored; \
         backup snapshot kept at {} (cause: {cause})",
        moved.len(),
        backup_dir.display()
    ))
}

fn clear_target_except_backup(target: &Path) {
    let Ok(rd) = std::fs::read_dir(target) else {
        return;
    };
    for entry in rd.flatten() {
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with(PRE_IMPORT_PREFIX)
        {
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

    #[test]
    fn restores_live_contents_when_staged_root_is_not_a_directory() {
        let target = TempDir::new().unwrap();
        fs::write(target.path().join("a.txt"), b"a").unwrap();
        fs::write(target.path().join("b.txt"), b"b").unwrap();
        fs::create_dir_all(target.path().join("sub")).unwrap();
        fs::write(target.path().join("sub/c.txt"), b"c").unwrap();
        let staged = TempDir::new().unwrap();
        let staged_file = staged.path().join("staged-file");
        fs::write(&staged_file, b"x").unwrap();

        let err = replace_with_staged(target.path(), &staged_file).unwrap_err();
        assert!(!err.to_string().is_empty());

        assert_eq!(fs::read(target.path().join("a.txt")).unwrap(), b"a");
        assert_eq!(fs::read(target.path().join("b.txt")).unwrap(), b"b");
        assert_eq!(fs::read(target.path().join("sub/c.txt")).unwrap(), b"c");
        assert!(!target.path().read_dir().unwrap().any(|e| {
            e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(PRE_IMPORT_PREFIX)
        }));
    }

    #[test]
    fn restores_live_contents_when_staged_copy_fails() {
        let target = TempDir::new().unwrap();
        fs::write(target.path().join("a.txt"), b"a").unwrap();
        fs::write(target.path().join("b.txt"), b"b").unwrap();
        fs::create_dir_all(target.path().join("sub")).unwrap();
        fs::write(target.path().join("sub/c.txt"), b"c").unwrap();
        let staged = TempDir::new().unwrap();
        fs::write(staged.path().join("n1.txt"), b"n1").unwrap();
        fs::write(staged.path().join("n2.txt"), b"n2").unwrap();

        let copy = |from: &Path, to: &Path| -> CoreResult<()> {
            if from.file_name().map(|n| n == "n2.txt").unwrap_or(false) {
                return Err(CoreError::Other("injected".into()));
            }
            path_util::copy_path(from, to)
        };

        let err = replace_with_staged_using(target.path(), staged.path(), copy).unwrap_err();
        assert!(err.to_string().contains("injected"), "got {err}");

        assert_eq!(fs::read(target.path().join("a.txt")).unwrap(), b"a");
        assert_eq!(fs::read(target.path().join("b.txt")).unwrap(), b"b");
        assert_eq!(fs::read(target.path().join("sub/c.txt")).unwrap(), b"c");
        assert!(!target.path().join("n1.txt").exists());
        assert!(!target.path().join("n2.txt").exists());
        assert!(!target.path().read_dir().unwrap().any(|e| {
            e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(PRE_IMPORT_PREFIX)
        }));
    }

    #[test]
    fn keeps_snapshot_when_restore_is_incomplete() {
        let target = TempDir::new().unwrap();
        fs::write(target.path().join("a.txt"), b"a").unwrap();
        fs::write(target.path().join("b.txt"), b"b").unwrap();
        fs::create_dir_all(target.path().join("sub")).unwrap();
        fs::write(target.path().join("sub/c.txt"), b"c").unwrap();
        let staged = TempDir::new().unwrap();
        fs::write(staged.path().join("n1.txt"), b"n1").unwrap();

        let copy = |_from: &Path, _to: &Path| -> CoreResult<()> {
            let backup = std::fs::read_dir(target.path())
                .unwrap()
                .find_map(|entry| {
                    let entry = entry.unwrap();
                    if entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with(PRE_IMPORT_PREFIX)
                    {
                        Some(entry.path())
                    } else {
                        None
                    }
                })
                .expect("snapshot present");
            fs::remove_file(backup.join("a.txt")).unwrap();
            Err(CoreError::Other("injected".into()))
        };

        let err = replace_with_staged_using(target.path(), staged.path(), copy).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("injected"), "got {msg}");
        assert!(msg.contains("rollback"), "got {msg}");

        let backup_name = std::fs::read_dir(target.path())
            .unwrap()
            .find_map(|entry| {
                let entry = entry.unwrap();
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(PRE_IMPORT_PREFIX) {
                    Some(name)
                } else {
                    None
                }
            })
            .expect("snapshot kept");
        let backup_dir = target.path().join(&backup_name);
        assert!(backup_dir.exists());
        assert!(msg.contains(&backup_dir.display().to_string()), "got {msg}");

        assert_eq!(fs::read(target.path().join("b.txt")).unwrap(), b"b");
        assert!(!target.path().join("a.txt").exists());
    }

    #[test]
    fn rollback_keeps_other_pre_import_snapshots() {
        let target = TempDir::new().unwrap();
        fs::write(target.path().join("a.txt"), b"a").unwrap();
        fs::write(target.path().join("b.txt"), b"b").unwrap();
        fs::create_dir_all(target.path().join("sub")).unwrap();
        fs::write(target.path().join("sub/c.txt"), b"c").unwrap();
        let leftover = target.path().join(".onq-pre-import-1");
        fs::create_dir_all(&leftover).unwrap();
        fs::write(leftover.join("keep.txt"), b"keep").unwrap();

        let staged = TempDir::new().unwrap();
        fs::write(staged.path().join("n1.txt"), b"n1").unwrap();

        let copy = |_from: &Path, _to: &Path| -> CoreResult<()> {
            Err(CoreError::Other("injected".into()))
        };

        let err = replace_with_staged_using(target.path(), staged.path(), copy).unwrap_err();
        assert!(err.to_string().contains("injected"), "got {err}");

        assert_eq!(fs::read(target.path().join("a.txt")).unwrap(), b"a");
        assert_eq!(fs::read(target.path().join("b.txt")).unwrap(), b"b");
        assert_eq!(fs::read(target.path().join("sub/c.txt")).unwrap(), b"c");
        assert_eq!(fs::read(leftover.join("keep.txt")).unwrap(), b"keep");
    }
}
