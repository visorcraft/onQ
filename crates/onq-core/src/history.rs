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
/// If the timestamp-derived name already exists, the candidate is bumped by
/// +1ns (and repeats) until a free name is found. This guards against coarse
/// system clocks (Windows/macOS tick granularity), NTP backward steps, and
/// rapid programmatic saves all landing on the same fixed-width timestamp;
/// without it, `atomic_write` would silently rename over the earlier snapshot.
///
/// Returns silently on success; surfaces IO errors as `CoreError::Io`.
pub fn snapshot(vault: &Vault, id: &PromptId, body: &str) -> CoreResult<()> {
    let dir = snapshot_dir(vault, id);
    std::fs::create_dir_all(&dir)?;
    let path = unique_snapshot_path(&dir, Utc::now());
    crate::vault::atomic_write(&path, body.as_bytes())
}

/// Return a path under `dir` whose name is free of existing files.
///
/// Candidates start at `at` and step forward one nanosecond at a time, so a
/// colliding name is never overwritten. Caps at 10_000 iterations; on
/// exhaustion returns the last candidate so the function stays total.
fn unique_snapshot_path(dir: &Path, at: DateTime<Utc>) -> PathBuf {
    let mut path = dir.join(snapshot_file_name(at));
    for i in 0..10_000i64 {
        let candidate = at + Duration::nanoseconds(i);
        path = dir.join(snapshot_file_name(candidate));
        if !path.exists() {
            return path;
        }
    }
    path
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
    // Decorate once (one timestamp parse per path), sort, undecorate. Tuple
    // Ord gives the old order: unparseable names first, then chronological,
    // ties by path.
    let mut decorated: Vec<(Option<DateTime<Utc>>, PathBuf)> = paths
        .into_iter()
        .map(|path| (timestamp_from_filename(&path), path))
        .collect();
    decorated.sort();
    Ok(decorated.into_iter().map(|(_, path)| path).collect())
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

/// Parse the timestamp embedded in a snapshot filename.
///
/// Supports both name formats: legacy RFC 3339 names with `:` time
/// separators (e.g. `2026-11-22T10:30:09.123456789Z.md`) and the portable
/// format with `-` time separators and a fixed-width 9-digit fractional
/// part (e.g. `2026-11-22T10-30-09.123456789Z.md`). This function is
/// allocation-free and runs once per snapshot on every save.
fn timestamp_from_filename(path: &Path) -> Option<DateTime<Utc>> {
    let stem = path.file_stem()?.to_str()?;
    if stem.contains(':') {
        // Legacy RFC 3339 name.
        return DateTime::parse_from_rfc3339(stem)
            .ok()
            .map(|dt| dt.with_timezone(&Utc));
    }
    let (date, _time) = stem.split_once('T')?;
    if stem.len() > 64 {
        return None; // no valid RFC 3339 timestamp is that long
    }
    // Copy the stem byte for byte, restoring ':' after 'T' (the date part
    // keeps its dashes). Stack-allocated; no String/format!.
    let mut buf = [0u8; 64];
    buf[..stem.len()].copy_from_slice(stem.as_bytes());
    for byte in &mut buf[date.len() + 1..stem.len()] {
        if *byte == b'-' {
            *byte = b':';
        }
    }
    let rewritten = std::str::from_utf8(&buf[..stem.len()]).ok()?;
    DateTime::parse_from_rfc3339(rewritten)
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

    #[test]
    fn unique_snapshot_path_skips_existing_name() {
        let dir = TempDir::new().unwrap();
        let t = Utc
            .with_ymd_and_hms(2026, 11, 22, 10, 30, 9)
            .unwrap()
            .with_nanosecond(123_456_789)
            .unwrap();
        let existing = dir.path().join(snapshot_file_name(t));
        std::fs::write(&existing, b"x").unwrap();
        let returned = unique_snapshot_path(dir.path(), t);
        assert_ne!(returned, existing);
        let dt = timestamp_from_filename(&returned).expect("returned path should parse");
        assert_eq!(dt, t + Duration::nanoseconds(1));
        assert!(existing < returned);
    }

    #[test]
    fn unique_snapshot_path_rolls_over_nanosecond_boundary() {
        let dir = TempDir::new().unwrap();
        let t = Utc
            .with_ymd_and_hms(2026, 11, 22, 10, 30, 9)
            .unwrap()
            .with_nanosecond(999_999_999)
            .unwrap();
        let existing = dir.path().join(snapshot_file_name(t));
        std::fs::write(&existing, b"x").unwrap();
        let returned = unique_snapshot_path(dir.path(), t);
        let dt = timestamp_from_filename(&returned).expect("returned path should parse");
        assert_eq!(dt, t + Duration::nanoseconds(1));
        let stem = returned.file_stem().unwrap().to_str().unwrap();
        assert!(stem.ends_with('Z'));
        let frac = stem.rsplit('.').next().unwrap().strip_suffix('Z').unwrap();
        assert_eq!(frac.len(), 9);
        assert!(frac.chars().all(|c| c.is_ascii_digit()));
        assert!(existing < returned);
    }

    #[test]
    fn unique_snapshot_path_returns_base_when_free() {
        let dir = TempDir::new().unwrap();
        let t = Utc
            .with_ymd_and_hms(2026, 11, 22, 10, 30, 9)
            .unwrap()
            .with_nanosecond(123_456_789)
            .unwrap();
        let returned = unique_snapshot_path(dir.path(), t);
        assert_eq!(returned, dir.path().join(snapshot_file_name(t)));
    }

    #[test]
    fn timestamp_from_filename_accepts_legacy_and_portable_names() {
        let expected = Utc.with_ymd_and_hms(2026, 11, 22, 10, 30, 9).unwrap()
            + Duration::nanoseconds(123_456_789);
        for name in [
            "2026-11-22T10-30-09.123456789Z.md",
            "2026-11-22T10:30:09.123456789Z.md",
            "2026-11-22T10:30:09.123456789+00:00.md",
        ] {
            assert_eq!(timestamp_from_filename(Path::new(name)), Some(expected));
        }
        let expected_no_frac = Utc.with_ymd_and_hms(2026, 11, 22, 10, 30, 9).unwrap();
        assert_eq!(
            timestamp_from_filename(Path::new("2026-11-22T10-30-09Z.md")),
            Some(expected_no_frac)
        );
    }

    #[test]
    fn timestamp_from_filename_rejects_non_timestamps() {
        let name_100 = "T-a-".repeat(25); // 100 chars of letters and dashes
        assert_eq!(name_100.len(), 100);
        for name in [
            "notes.md",
            "2026-11-22.md",
            "2026-13-45T99-99-99Z.md",
            "2026-11-22T10:30-09Z.md",
            "T10-30-09.md",
            &name_100,
        ] {
            assert_eq!(timestamp_from_filename(Path::new(name)), None);
        }
    }

    #[test]
    fn timestamp_from_filename_matches_previous_implementation() {
        fn legacy_timestamp_from_filename(path: &Path) -> Option<DateTime<Utc>> {
            let stem = path.file_stem()?.to_str()?;
            if let Ok(dt) = DateTime::parse_from_rfc3339(stem) {
                return Some(dt.with_timezone(&Utc));
            }
            let (date, time) = stem.split_once('T')?;
            if time.contains(':') {
                return None;
            }
            DateTime::parse_from_rfc3339(&format!("{}T{}", date, time.replace('-', ":")))
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        }

        let name_100 = "T-a-".repeat(25);
        let name_64 = "1".repeat(63) + "-";
        let name_65 = "1".repeat(64) + "-";
        assert_eq!(name_64.len(), 64);
        assert_eq!(name_65.len(), 65);

        let names = [
            "2026-11-22T10-30-09.123456789Z.md",
            "2026-11-22T10:30:09.123456789Z.md",
            "2026-11-22T10:30:09.123456789+00:00.md",
            "2026-11-22T10-30-09Z.md",
            "notes.md",
            "2026-11-22.md",
            "2026-13-45T99-99-99Z.md",
            "2026-11-22T10:30-09Z.md",
            "T10-30-09.md",
            &name_100,
            "2026-11-22T10-30-09+00-00.md",
            "2026-11-22T10-30-09.5Z.md",
            "2026-11-22T10:30:09.md",
            "2026-11-22T10-30-09Z-é.md",
            "2026-11-22T10-30-09.123456789-05-00.md",
            &name_64,
            &name_65,
        ];
        for name in names {
            assert_eq!(
                timestamp_from_filename(Path::new(name)),
                legacy_timestamp_from_filename(Path::new(name)),
                "mismatch for {name:?}"
            );
        }
    }

    #[test]
    fn list_snapshots_sorts_mixed_formats_with_unparseable_first() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let id = PromptId::new();
        let snap_dir = snapshot_dir(&vault, &id);
        std::fs::create_dir_all(&snap_dir).unwrap();
        let portable_0900 =
            snapshot_file_name(Utc.with_ymd_and_hms(2026, 11, 22, 9, 0, 0).unwrap());
        let legacy_1000 = "2026-11-22T10:00:00Z.md";
        let portable_1100 =
            snapshot_file_name(Utc.with_ymd_and_hms(2026, 11, 22, 11, 0, 59).unwrap());
        std::fs::write(snap_dir.join("notes.md"), b"x").unwrap();
        std::fs::write(snap_dir.join(&portable_0900), b"x").unwrap();
        std::fs::write(snap_dir.join(legacy_1000), b"x").unwrap();
        std::fs::write(snap_dir.join(&portable_1100), b"x").unwrap();
        std::fs::write(snap_dir.join("skip.txt"), b"x").unwrap();
        let paths = list_snapshots(&vault, &id).unwrap();
        let names: Vec<String> = paths
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            names,
            vec![
                "notes.md".to_string(),
                portable_0900,
                legacy_1000.to_string(),
                portable_1100,
            ]
        );
    }

    #[test]
    fn prune_older_than_removes_old_snapshots_in_both_formats() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let id_a = PromptId::new();
        let id_b = PromptId::new();
        let old = Utc::now() - Duration::days(40);
        let fresh = Utc::now();
        let old_b = Utc::now() - Duration::days(31);

        let dir_a = snapshot_dir(&vault, &id_a);
        std::fs::create_dir_all(&dir_a).unwrap();
        std::fs::write(dir_a.join(snapshot_file_name(old)), b"x").unwrap();
        std::fs::write(
            dir_a.join(format!(
                "{}.md",
                fresh.to_rfc3339_opts(SecondsFormat::Nanos, true)
            )),
            b"x",
        )
        .unwrap();

        let dir_b = snapshot_dir(&vault, &id_b);
        std::fs::create_dir_all(&dir_b).unwrap();
        std::fs::write(dir_b.join(snapshot_file_name(old_b)), b"x").unwrap();
        std::fs::write(dir_b.join("keep.txt"), b"x").unwrap();

        let history_root = vault.root.join(".onq").join("history");
        std::fs::write(history_root.join("stray.md"), b"x").unwrap();

        assert_eq!(prune_older_than(&vault, 30).unwrap(), 2);

        let mut remaining_a: Vec<String> = std::fs::read_dir(&dir_a)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        let mut remaining_b: Vec<String> = std::fs::read_dir(&dir_b)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        remaining_a.sort();
        remaining_b.sort();
        assert!(history_root.join("stray.md").exists());
        assert_eq!(
            remaining_a,
            vec![format!(
                "{}.md",
                fresh.to_rfc3339_opts(SecondsFormat::Nanos, true)
            )]
        );
        assert_eq!(remaining_b, vec!["keep.txt".to_string()]);
    }
}
