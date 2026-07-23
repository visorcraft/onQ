//! Local append-only security audit log under `.onq/audit.log` (JSONL).

use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::CoreResult;
use crate::vault::Vault;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditEvent {
    pub at: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Append one audit event. Creates parent dirs as needed.
pub fn append(vault: &Vault, kind: &str, detail: Option<&str>) -> CoreResult<()> {
    let path = audit_path(vault);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let event = AuditEvent {
        at: Utc::now().to_rfc3339(),
        kind: kind.into(),
        detail: detail.map(|s| s.to_string()),
    };
    let mut line = serde_json::to_string(&event)?;
    line.push('\n');
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    f.write_all(line.as_bytes())?;
    Ok(())
}

/// Read the last `limit` events (newest last in file order, returned newest-first).
pub fn read_recent(vault: &Vault, limit: usize) -> CoreResult<Vec<AuditEvent>> {
    let path = audit_path(vault);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(&path)?;
    let mut events: Vec<AuditEvent> = raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    if events.len() > limit {
        events = events.split_off(events.len() - limit);
    }
    events.reverse();
    Ok(events)
}

pub fn audit_path(vault: &Vault) -> PathBuf {
    vault.root.join(".onq").join("audit.log")
}

/// Pure helper: whether a backup reminder should show.
pub fn should_remind_backup(
    last_backup_rfc3339: &str,
    remind_days: u32,
    now: chrono::DateTime<Utc>,
) -> bool {
    if remind_days == 0 {
        return false;
    }
    if last_backup_rfc3339.trim().is_empty() {
        return true;
    }
    let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(last_backup_rfc3339) else {
        return true;
    };
    let last = parsed.with_timezone(&Utc);
    now.signed_duration_since(last).num_days() >= remind_days as i64
}

/// Validate path is under vault history (best-effort path traversal guard).
pub fn path_under(root: &Path, candidate: &Path) -> bool {
    let Ok(root) = root.canonicalize() else {
        return false;
    };
    let Ok(cand) = candidate.canonicalize() else {
        return false;
    };
    cand.starts_with(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn append_and_read_roundtrip() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        append(&vault, "vault_unlock", Some("test")).unwrap();
        append(&vault, "vault_lock", None).unwrap();
        let events = read_recent(&vault, 10).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, "vault_lock");
        assert_eq!(events[1].kind, "vault_unlock");
    }

    #[test]
    fn backup_remind_logic() {
        let now = Utc::now();
        assert!(should_remind_backup("", 7, now));
        assert!(!should_remind_backup(&now.to_rfc3339(), 7, now));
        assert!(!should_remind_backup("x", 0, now));
    }
}
