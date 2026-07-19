//! Reconcile a prompt's vault file with its indexed database row.

use chrono::Utc;
use mongreldb_core::{query::Condition, Query, Value};
use sha2::{Digest, Sha256};

use crate::db::Db;
use crate::error::{CoreError, CoreResult};
use crate::merge::{self, MergeOutcome};
use crate::schema::col;
use crate::sync_state::{self, Sidecar};
use crate::ulid::PromptId;
use crate::vault::{atomic_write, Vault};

pub enum ReconcileOutcome {
    Noop,
    Ingested,
    Merged(MergeOutcome),
}

/// Compare the prompt on disk with its sidecar and current database body.
///
/// A missing sidecar is treated as a fresh external file: its disk hash is
/// recorded as `ours_sha`, while `theirs_sha` remains empty so the file takes
/// the ingest path. Reconciliation returns the work for the caller to apply to
/// the database; this function persists the clean sidecar and a content-addressed
/// base snapshot for an ingest.
pub fn reconcile(vault: &Vault, db: &Db, id: &PromptId) -> CoreResult<ReconcileOutcome> {
    let disk = vault.read(id)?;
    let disk_hash = sha256(&disk.body);
    let mut sidecar = sync_state::read(&vault.root, id)?
        .unwrap_or_else(|| fresh_sidecar(id.clone(), disk_hash.clone()));
    let db_body = read_db_body(db, id)?;

    if !sidecar.theirs_sha.is_empty() && disk_hash == sidecar.theirs_sha {
        return Ok(ReconcileOutcome::Noop);
    }

    let db_hash = match db_body.as_ref() {
        Some(body) => sha256(body),
        None => String::new(),
    };

    let fresh_sidecar = sidecar.theirs_sha.is_empty();
    let db_unchanged = !db_hash.is_empty() && sidecar.ours_sha == db_hash;
    if fresh_sidecar || db_unchanged {
        write_base(vault, &disk_hash, &disk.body)?;
        sidecar.base_sha = disk_hash.clone();
        sidecar.ours_sha = disk_hash.clone();
        sidecar.theirs_sha = disk_hash;
        sidecar.last_synced_at = Utc::now().to_rfc3339();
        sync_state::write(&vault.root, &sidecar)?;
        return Ok(ReconcileOutcome::Ingested);
    }

    let ours = db_body.ok_or_else(|| CoreError::Db(format!("prompt {id} not found")))?;
    let ours_hash = sha256(&ours);
    let base = if sidecar.base_sha == ours_hash {
        ours.clone()
    } else if sidecar.base_sha == disk_hash {
        disk.body.clone()
    } else {
        read_base(vault, &sidecar.base_sha)?
    };
    let merged = merge::three_way(&base, &ours, &disk.body)?;
    Ok(ReconcileOutcome::Merged(merged))
}

fn fresh_sidecar(id: PromptId, disk_hash: String) -> Sidecar {
    Sidecar {
        id,
        base_sha: disk_hash.clone(),
        ours_sha: disk_hash,
        theirs_sha: String::new(),
        last_synced_at: String::new(),
        external_edit_count: 0,
        tombstone: false,
    }
}

fn write_base(vault: &Vault, sha: &str, body: &str) -> CoreResult<()> {
    atomic_write(&base_path(vault, sha)?, body.as_bytes())
}

fn read_base(vault: &Vault, sha: &str) -> CoreResult<String> {
    let path = base_path(vault, sha)?;
    std::fs::read_to_string(&path).map_err(|error| {
        CoreError::Merge(format!(
            "read base snapshot {} for {sha}: {error}",
            path.display()
        ))
    })
}

fn base_path(vault: &Vault, sha: &str) -> CoreResult<std::path::PathBuf> {
    if sha.len() != 64 || !sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(CoreError::Merge(format!(
            "invalid base snapshot SHA: {sha}"
        )));
    }
    Ok(vault.root.join(".onq/bases").join(format!("{sha}.txt")))
}

fn read_db_body(db: &Db, id: &PromptId) -> CoreResult<Option<String>> {
    let rows = db
        .handle()
        .query_for_current_principal(
            "prompts",
            &Query {
                conditions: vec![Condition::Pk(id.as_str().as_bytes().to_vec())],
                ..Default::default()
            },
            None,
        )
        .map_err(|error| CoreError::Db(format!("read prompt {id}: {error}")))?;

    let Some(row) = rows.into_iter().next() else {
        return Ok(None);
    };
    match row.columns.get(&col::PROMPTS_BODY) {
        Some(Value::Bytes(body)) => String::from_utf8(body.clone())
            .map(Some)
            .map_err(|error| CoreError::Db(format!("prompt {id} body is not UTF-8: {error}"))),
        _ => Err(CoreError::Db(format!(
            "prompt {id} body is missing or has the wrong type"
        ))),
    }
}

fn sha256(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongreldb_core::Value;
    use tempfile::TempDir;

    #[test]
    fn unchanged_disk_hash_is_noop() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let mut prompt = vault.new_prompt("Unchanged").unwrap();
        prompt.body = "same body".into();
        vault.write(&prompt).unwrap();

        let db = Db::open(dir.path(), "test-pass").unwrap();
        insert_prompt(&db, &prompt.fm.id, &prompt.body);

        let hash = sha256(&prompt.body);
        let sidecar = Sidecar {
            id: prompt.fm.id.clone(),
            base_sha: hash.clone(),
            ours_sha: hash.clone(),
            theirs_sha: hash,
            last_synced_at: "2026-07-19T00:00:00Z".into(),
            external_edit_count: 0,
            tombstone: false,
        };
        sync_state::write(&vault.root, &sidecar).unwrap();

        let outcome = reconcile(&vault, &db, &prompt.fm.id).unwrap();
        assert!(matches!(outcome, ReconcileOutcome::Noop));
        assert_eq!(
            sync_state::read(&vault.root, &prompt.fm.id)
                .unwrap()
                .unwrap(),
            sidecar
        );
    }

    #[test]
    fn missing_sidecar_is_ingested_and_base_is_archived() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let mut prompt = vault.new_prompt("Fresh").unwrap();
        prompt.body = "fresh body".into();
        vault.write(&prompt).unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();

        let outcome = reconcile(&vault, &db, &prompt.fm.id).unwrap();
        assert!(matches!(outcome, ReconcileOutcome::Ingested));

        let hash = sha256(&prompt.body);
        let sidecar = sync_state::read(&vault.root, &prompt.fm.id)
            .unwrap()
            .unwrap();
        assert_eq!(sidecar.base_sha, hash);
        assert_eq!(sidecar.ours_sha, hash);
        assert_eq!(sidecar.theirs_sha, hash);
        assert_eq!(read_base(&vault, &hash).unwrap(), prompt.body);
    }

    #[test]
    fn existing_prompt_db_unchanged_disk_changed_returns_ingested() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let mut prompt = vault.new_prompt("External").unwrap();
        prompt.body = "line 1\nline 2\nline 3\n".into();
        vault.write(&prompt).unwrap();

        let db = Db::open(dir.path(), "test-pass").unwrap();
        insert_prompt(&db, &prompt.fm.id, &prompt.body);

        let hash = sha256(&prompt.body);
        write_base(&vault, &hash, &prompt.body).unwrap();
        let sidecar = Sidecar {
            id: prompt.fm.id.clone(),
            base_sha: hash.clone(),
            ours_sha: hash.clone(),
            theirs_sha: hash,
            last_synced_at: "2026-07-19T00:00:00Z".into(),
            external_edit_count: 0,
            tombstone: false,
        };
        sync_state::write(&vault.root, &sidecar).unwrap();

        // External editor mutates only the disk file.
        prompt.body = "line 1\nline TWO\nline 3\n".into();
        vault.write(&prompt).unwrap();

        let outcome = reconcile(&vault, &db, &prompt.fm.id).unwrap();
        assert!(
            matches!(outcome, ReconcileOutcome::Ingested),
            "expected Ingested"
        );

        // Sidecar is rewound to the new disk hash; ours now matches the
        // external edit so subsequent reconciles are Noop until the next
        // external edit or DB change.
        let new_hash = sha256(&prompt.body);
        let updated = sync_state::read(&vault.root, &prompt.fm.id)
            .unwrap()
            .unwrap();
        assert_eq!(updated.base_sha, new_hash);
        assert_eq!(updated.ours_sha, new_hash);
        assert_eq!(updated.theirs_sha, new_hash);
        assert_eq!(read_base(&vault, &new_hash).unwrap(), prompt.body);
    }

    #[test]
    fn both_db_and_disk_changed_returns_three_way_merge() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let mut prompt = vault.new_prompt("Both").unwrap();
        prompt.body = "line 1\nline 2\nline 3\n".into();
        vault.write(&prompt).unwrap();

        let db = Db::open(dir.path(), "test-pass").unwrap();
        insert_prompt(&db, &prompt.fm.id, &prompt.body);

        let hash = sha256(&prompt.body);
        write_base(&vault, &hash, &prompt.body).unwrap();
        let sidecar = Sidecar {
            id: prompt.fm.id.clone(),
            base_sha: hash.clone(),
            ours_sha: hash.clone(),
            theirs_sha: hash,
            last_synced_at: "2026-07-19T00:00:00Z".into(),
            external_edit_count: 0,
            tombstone: false,
        };
        sync_state::write(&vault.root, &sidecar).unwrap();

        // User edits inside the app (DB) while the file is edited externally
        // (disk) in a conflicting region — 3-way merge must be invoked.
        let db_body = "line 1\nDB line 2\nline 3\n";
        upsert_prompt(&db, &prompt.fm.id, b"Both", db_body);
        prompt.body = "line 1\nDISK line 2\nline 3\n".into();
        vault.write(&prompt).unwrap();

        let outcome = reconcile(&vault, &db, &prompt.fm.id).unwrap();
        match outcome {
            ReconcileOutcome::Merged(MergeOutcome::Conflicted { text }) => {
                assert!(text.contains("<<<<<<<"));
                assert!(text.contains("======="));
                assert!(text.contains(">>>>>>>"));
            }
            ReconcileOutcome::Merged(MergeOutcome::Clean(_)) => {
                panic!("expected conflicted merge, got clean merge");
            }
            _ => panic!("expected Merged(Conflicted)"),
        }
    }

    fn insert_prompt(db: &Db, id: &PromptId, body: &str) {
        upsert_prompt(db, id, b"Unchanged", body);
    }

    fn upsert_prompt(db: &Db, id: &PromptId, title: &[u8], body: &str) {
        db.handle()
            .transaction_for_current_principal(|tx| {
                tx.put(
                    "prompts",
                    vec![
                        (
                            col::PROMPTS_ID,
                            Value::Bytes(id.as_str().as_bytes().to_vec()),
                        ),
                        (col::PROMPTS_TITLE, Value::Bytes(title.to_vec())),
                        (col::PROMPTS_BODY, Value::Bytes(body.as_bytes().to_vec())),
                        (col::PROMPTS_TAGS, Value::Json(b"[]".to_vec())),
                        (col::PROMPTS_FAVORITE, Value::Bool(false)),
                        (col::PROMPTS_LOCKED, Value::Bool(false)),
                        (col::PROMPTS_CHAR, Value::Int64(body.len() as i64)),
                        (col::PROMPTS_CREATED, Value::Int64(0)),
                        (col::PROMPTS_UPDATED, Value::Int64(0)),
                        (col::PROMPTS_EMBED, Value::Embedding(vec![0.0; 384])),
                    ],
                )
            })
            .unwrap();
    }
}
