//! History snapshots. Each prompt edit is archived as
//! `<vault>/.onq/history/<id>/<RFC3339-timestamp>.md` so users can
//! diff/restore prior versions.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Utc};

use crate::error::{CoreError, CoreResult};
use crate::ulid::PromptId;
use crate::vault::Vault;

/// Write a snapshot of `body` for prompt `id` at the current time.
///
/// Returns silently on success; surfaces IO errors as `CoreError::Io`.
pub fn snapshot(vault: &Vault, id: &PromptId, body: &str) -> CoreResult<()> {
    let dir = snapshot_dir(vault, id);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.md", Utc::now().to_rfc3339()));
    crate::vault::atomic_write(&path, body.as_bytes())
}

/// List all snapshot files for `id`, sorted ascending by filename
/// (RFC3339 lexicographic order matches chronological order).
pub fn list_snapshots(vault: &Vault, id: &PromptId) -> CoreResult<Vec<PathBuf>> {
    let dir = snapshot_dir(vault, id);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

/// Read snapshot body text from an absolute path under the vault history tree.
pub fn read_snapshot(path: &Path) -> CoreResult<String> {
    std::fs::read_to_string(path).map_err(CoreError::from)
}

/// Restore a prompt's body from a history snapshot, writing a new current file.
/// Does not re-snapshot the pre-restore body (caller may write via Vault first).
pub fn restore_body(vault: &Vault, id: &PromptId, snapshot_path: &Path) -> CoreResult<String> {
    let body = read_snapshot(snapshot_path)?;
    let mut prompt = vault.read(id)?;
    prompt.body = body.clone();
    // Skip history prune churn: write without new snapshot of restored content
    // by using vault write which does snapshot the restored body (desired).
    vault.write(&prompt)?;
    Ok(body)
}

/// Delete every snapshot in every prompt's history folder whose filename
/// timestamp is older than `days` days. Returns the number of files removed.
pub fn prune_older_than(vault: &Vault, days: u32) -> CoreResult<usize> {
    let root = history_root(vault);
    if !root.exists() {
        return Ok(0);
    }
    let cutoff = Utc::now() - Duration::days(days as i64);
    let mut removed = 0usize;
    walk_history(&root, &mut |path| {
        let Some(ts) = timestamp_from_filename(path) else {
            return;
        };
        if ts < cutoff && std::fs::remove_file(path).is_ok() {
            removed += 1;
        }
    })?;
    Ok(removed)
}

fn history_root(vault: &Vault) -> PathBuf {
    vault.root.join(".onq").join("history")
}

fn snapshot_dir(vault: &Vault, id: &PromptId) -> PathBuf {
    history_root(vault).join(id.as_str())
}

fn timestamp_from_filename(path: &Path) -> Option<DateTime<Utc>> {
    let stem = path.file_stem()?.to_str()?;
    DateTime::parse_from_rfc3339(stem)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn walk_history(root: &Path, visit: &mut dyn FnMut(&Path)) -> CoreResult<()> {
    for prompt_dir in std::fs::read_dir(root)? {
        let prompt_dir = prompt_dir?;
        if !prompt_dir.file_type()?.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(prompt_dir.path())? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                visit(&entry.path());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn snapshot_writes_file() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let id = PromptId::new();
        snapshot(&vault, &id, "first body").unwrap();
        let paths = list_snapshots(&vault, &id).unwrap();
        assert_eq!(paths.len(), 1);
        let content = std::fs::read_to_string(&paths[0]).unwrap();
        assert_eq!(content, "first body");
    }

    #[test]
    fn list_snapshots_returns_sorted() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let id = PromptId::new();
        let snap_dir = snapshot_dir(&vault, &id);
        std::fs::create_dir_all(&snap_dir).unwrap();
        for name in [
            "2026-07-18T10:00:00Z.md",
            "2026-07-17T09:00:00Z.md",
            "2026-07-19T11:00:00Z.md",
        ] {
            std::fs::write(snap_dir.join(name), b"x").unwrap();
        }
        let paths = list_snapshots(&vault, &id).unwrap();
        assert_eq!(paths.len(), 3);
        let names: Vec<String> = paths
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            names,
            vec![
                "2026-07-17T09:00:00Z.md",
                "2026-07-18T10:00:00Z.md",
                "2026-07-19T11:00:00Z.md",
            ]
        );
    }
}
