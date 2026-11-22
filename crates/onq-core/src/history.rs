//! History snapshots. Each prompt edit is archived as
//! `<vault>/.onq/history/<id>/<timestamp>.md` so users can
//! diff/restore prior versions.
//!
//! The timestamp is RFC 3339 UTC with `-` time separators, a fixed-width
//! 9-digit fractional part, and a `Z` suffix (e.g.
//! `2026-11-22T10-30-09.123456789Z`), chosen so the name is valid on
//! Windows/macOS/Linux and sorts chronologically. Legacy names with `:`
//! time separators remain readable.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, SecondsFormat, Utc};

use crate::error::{CoreError, CoreResult};
use crate::ulid::PromptId;
use crate::vault::Vault;

/// Write a snapshot of `body` for prompt `id` at the current time.
///
/// Returns silently on success; surfaces IO errors as `CoreError::Io`.
pub fn snapshot(vault: &Vault, id: &PromptId, body: &str) -> CoreResult<()> {
    let dir = snapshot_dir(vault, id);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(snapshot_file_name(Utc::now()));
    crate::vault::atomic_write(&path, body.as_bytes())
}

/// Filesystem-portable snapshot name: RFC 3339 UTC with `-` time
/// separators, a fixed-width 9-digit fractional part, and a `Z` suffix
/// (no `:`, no `+`), so that lexicographic file order equals
/// chronological order.
fn snapshot_file_name(at: DateTime<Utc>) -> String {
    format!(
        "{}.md",
        at.to_rfc3339_opts(SecondsFormat::Nanos, true)
            .replace(':', "-")
    )
}

/// List all snapshot files for `id`, sorted ascending chronologically by
/// parsed filename timestamp (falling back to path order for unparseable
/// names, so legacy `:` names and portable `-` names within the same hour
/// stay strictly chronological).
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
    paths.sort_by(|a, b| (timestamp_from_filename(a), a).cmp(&(timestamp_from_filename(b), b)));
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
    if let Ok(dt) = DateTime::parse_from_rfc3339(stem) {
        return Some(dt.with_timezone(&Utc)); // legacy ...+00:00 / ...Z
    }
    // Portable names use '-' as the time separator; restore ':' after 'T'.
    let (date, time) = stem.split_once('T')?;
    if time.contains(':') {
        return None;
    }
    DateTime::parse_from_rfc3339(&format!("{}T{}", date, time.replace('-', ":")))
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
    use chrono::{TimeZone, Timelike};
    use tempfile::TempDir;

    #[test]
    fn snapshot_file_name_is_filesystem_portable() {
        let ts = Utc
            .with_ymd_and_hms(2026, 11, 22, 10, 30, 9)
            .unwrap()
            .with_nanosecond(123_456_789)
            .unwrap();
        let name = snapshot_file_name(ts);
        assert_eq!(name, "2026-11-22T10-30-09.123456789Z.md");
        assert!(!name.contains(':'));
        assert!(name.chars().all(|c| c.is_ascii_alphanumeric()
            || c == '-'
            || c == '.'
            || c == 'T'
            || c == 'Z'));
    }

    #[test]
    fn snapshot_file_name_is_fixed_width_and_sorts_chronologically() {
        let base = Utc.with_ymd_and_hms(2026, 11, 22, 10, 30, 9).unwrap();
        let names: Vec<String> = [0, 1, 123_000_000, 999_999_999]
            .iter()
            .map(|&ns| snapshot_file_name(base.with_nanosecond(ns).unwrap()))
            .collect();
        let widths: Vec<usize> = names.iter().map(|n| n.len()).collect();
        assert!(widths.iter().all(|&w| w == widths[0]));
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(sorted, names);
        let earlier = Utc.with_ymd_and_hms(2026, 11, 22, 10, 30, 8).unwrap();
        let later = Utc.with_ymd_and_hms(2026, 11, 22, 10, 30, 9).unwrap();
        assert!(snapshot_file_name(earlier) < snapshot_file_name(later));
    }

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
        let ts = timestamp_from_filename(&paths[0]).unwrap();
        let name = paths[0].file_name().unwrap().to_str().unwrap();
        assert_eq!(name, snapshot_file_name(ts));
        assert!((Utc::now() - ts).num_seconds().abs() <= 10);
    }

    #[test]
    fn list_snapshots_orders_real_names_with_legacy_files() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let id = PromptId::new();
        let snap_dir = snapshot_dir(&vault, &id);
        std::fs::create_dir_all(&snap_dir).unwrap();
        let instants = [
            Utc.with_ymd_and_hms(2026, 7, 18, 10, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 18, 10, 0, 0)
                .unwrap()
                .with_nanosecond(500_000_000)
                .unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 19, 11, 0, 0).unwrap(),
        ];
        for ts in instants {
            std::fs::write(snap_dir.join(snapshot_file_name(ts)), b"x").unwrap();
        }
        std::fs::write(snap_dir.join("2026-07-16T08:00:00.25+00:00.md"), b"x").unwrap();
        let paths = list_snapshots(&vault, &id).unwrap();
        assert_eq!(paths.len(), 4);
        let names: Vec<String> = paths
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            names,
            vec![
                "2026-07-16T08:00:00.25+00:00.md",
                "2026-07-18T10-00-00.000000000Z.md",
                "2026-07-18T10-00-00.500000000Z.md",
                "2026-07-19T11-00-00.000000000Z.md",
            ]
        );
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

    #[test]
    fn list_snapshots_orders_legacy_and_portable_within_same_hour() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let id = PromptId::new();
        let snap_dir = snapshot_dir(&vault, &id);
        std::fs::create_dir_all(&snap_dir).unwrap();
        // Legacy ':' (ASCII 58) vs portable '-' (ASCII 45) within the same hour:
        // lexicographic sort would invert these, chronological sort must keep 10:05 before 10-10.
        let legacy_earlier = "2026-11-22T10:05:00+00:00.md";
        let portable_later =
            snapshot_file_name(Utc.with_ymd_and_hms(2026, 11, 22, 10, 10, 0).unwrap());
        std::fs::write(snap_dir.join(legacy_earlier), b"old").unwrap();
        std::fs::write(snap_dir.join(&portable_later), b"new").unwrap();
        let paths = list_snapshots(&vault, &id).unwrap();
        let names: Vec<String> = paths
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec![legacy_earlier.to_string(), portable_later]);
    }
}
