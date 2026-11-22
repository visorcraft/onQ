//! Per-prompt sync sidecar state.
//!
//! The sidecar stores the SHAs for the base, ours, and theirs versions of a
//! prompt along with sync metadata (last sync time, external edit count, and
//! tombstone flag). It lives next to the vault metadata under
//! `<vault>/.onq/state/<id>.json` and is written atomically.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CoreResult;
use crate::ulid::PromptId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Sidecar {
    pub id: PromptId,
    pub base_sha: String,
    pub ours_sha: String,
    pub theirs_sha: String,
    pub last_synced_at: String,
    pub external_edit_count: u32,
    pub tombstone: bool,
}

pub fn sidecar_path(vault: &Path, id: &PromptId) -> PathBuf {
    vault
        .join(".onq/state")
        .join(format!("{}.json", id.as_str()))
}

/// Read the sidecar for `id`.
///
/// A missing, corrupt, or non-UTF-8 sidecar is treated as absent so `reconcile`
/// takes the fresh-ingest path, where the disk content re-ingests over the db
/// row (same semantics as a missing sidecar). Genuine IO errors from the file
/// read still propagate.
pub fn read(vault: &Path, id: &PromptId) -> CoreResult<Option<Sidecar>> {
    let p = sidecar_path(vault, id);
    if !p.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(&p)?;
    let s = String::from_utf8_lossy(&bytes);
    Ok(serde_json::from_str(&s).ok())
}

pub fn write(vault: &Path, s: &Sidecar) -> CoreResult<()> {
    let path = sidecar_path(vault, &s.id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(s)?)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn roundtrip() {
        let dir = TempDir::new().unwrap();
        let id = PromptId::new();
        let s = Sidecar {
            id: id.clone(),
            base_sha: "abc123".into(),
            ours_sha: "def456".into(),
            theirs_sha: "ghi789".into(),
            last_synced_at: "2026-07-19T00:00:00Z".into(),
            external_edit_count: 3,
            tombstone: false,
        };
        write(dir.path(), &s).unwrap();
        let got = read(dir.path(), &id).unwrap().unwrap();
        assert_eq!(got, s);
    }

    #[test]
    fn read_missing_returns_none() {
        let dir = TempDir::new().unwrap();
        let id = PromptId::new();
        let got = read(dir.path(), &id).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn write_creates_parent_dir() {
        let dir = TempDir::new().unwrap();
        let id = PromptId::new();
        let s = Sidecar {
            id: id.clone(),
            base_sha: "b".into(),
            ours_sha: "o".into(),
            theirs_sha: "t".into(),
            last_synced_at: "2026-07-19T00:00:00Z".into(),
            external_edit_count: 0,
            tombstone: false,
        };
        let nested = dir.path().join("nested/vault");
        write(&nested, &s).unwrap();
        assert!(!read(&nested, &id).unwrap().unwrap().tombstone);
        assert_eq!(read(&nested, &id).unwrap().unwrap(), s);
    }

    #[test]
    fn read_tolerates_invalid_utf8() {
        let dir = TempDir::new().unwrap();
        let id = PromptId::new();
        let path = sidecar_path(dir.path(), &id);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"\xff\xfe{").unwrap();
        assert!(matches!(read(dir.path(), &id), Ok(None)));
    }

    #[test]
    fn read_tolerates_truncated_json() {
        let dir = TempDir::new().unwrap();
        let id = PromptId::new();
        let s = Sidecar {
            id: id.clone(),
            base_sha: "b".into(),
            ours_sha: "o".into(),
            theirs_sha: "t".into(),
            last_synced_at: "2026-07-19T00:00:00Z".into(),
            external_edit_count: 0,
            tombstone: false,
        };
        write(dir.path(), &s).unwrap();
        let path = sidecar_path(dir.path(), &id);
        std::fs::write(&path, "{\"id\": \"01A").unwrap();
        assert!(matches!(read(dir.path(), &id), Ok(None)));
    }

    #[test]
    fn read_tolerates_wrong_types() {
        let dir = TempDir::new().unwrap();
        let id = PromptId::new();
        let path = sidecar_path(dir.path(), &id);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "{\"id\": 5}").unwrap();
        assert!(matches!(read(dir.path(), &id), Ok(None)));
    }

    #[test]
    fn tombstone_survives_roundtrip() {
        let dir = TempDir::new().unwrap();
        let id = PromptId::new();
        let s = Sidecar {
            id: id.clone(),
            base_sha: "".into(),
            ours_sha: "".into(),
            theirs_sha: "".into(),
            last_synced_at: "2026-07-19T00:00:00Z".into(),
            external_edit_count: 0,
            tombstone: true,
        };
        write(dir.path(), &s).unwrap();
        let got = read(dir.path(), &id).unwrap().unwrap();
        assert!(got.tombstone);
        assert_eq!(got, s);
    }
}
