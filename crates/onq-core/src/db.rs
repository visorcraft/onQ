//! Thin wrapper around [`mongreldb_core::Database`] for the onQ vault.
//!
//! `Db::open` either opens an existing encrypted database at the standard
//! `search-index` subdirectory, or creates a new one. On every open (new or
//! existing), the [`crate::migrate::Migrator`] runs to apply any pending
//! schema migrations — including the initial "create all tables + seed
//! `app_state`" pass on a fresh vault. All access flows through `handle()`,
//! which exposes the underlying [`mongreldb_core::Database`] synchronously —
//! callers must wrap calls in `tokio::task::spawn_blocking` from async
//! contexts.

use std::path::Path;

use mongreldb_core::query::Query;
use mongreldb_core::{Database, Value};

use crate::error::{CoreError, CoreResult};
use crate::schema::col;

/// Wraps an open, encrypted MongrelDB instance bound to a onQ vault.
pub struct Db {
    inner: Database,
}

impl Db {
    /// Opens or creates an encrypted database at `<path>/.onq/search-index`,
    /// then runs every pending migration from [`crate::migrate::Migrator`].
    ///
    /// Sync (mongreldb-core is sync). The migrator is the single source of
    /// truth for what runs on open — table creation, the singleton
    /// `app_state` row, and any future schema additions all flow through it.
    pub fn open(vault_path: &Path, passphrase: &str) -> CoreResult<Self> {
        let root = vault_path.join(".onq").join("search-index");
        let inner = if root.exists() {
            Database::open_encrypted(&root, passphrase)
                .map_err(|e| CoreError::Db(format!("open {}: {e}", root.display())))?
        } else {
            std::fs::create_dir_all(&root)
                .map_err(|e| CoreError::Db(format!("mkdir {}: {e}", root.display())))?;
            Database::create_encrypted(&root, passphrase)
                .map_err(|e| CoreError::Db(format!("create {}: {e}", root.display())))?
        };
        crate::migrate::Migrator::new(&inner)
            .run()
            .map_err(|e| CoreError::Db(format!("migrate: {e}")))?;
        Ok(Self { inner })
    }

    /// Access the underlying [`mongreldb_core::Database`].
    pub fn handle(&self) -> &Database {
        &self.inner
    }

    /// Look up the singleton `app_state` row's cell for `key` and return it
    /// as a UTF-8 string. Returns an empty string when the row is missing or
    /// the column is empty — the same degrade-safely contract the existing
    /// `read_app_setting` helper in `commands.rs` follows, exposed on `Db`
    /// so Tauri commands don't have to know about the table name.
    pub fn get_app_setting(&self, key: &str) -> CoreResult<String> {
        let column_id = setting_column_for_key(key)?;
        let rows = self
            .inner
            .query_for_current_principal("app_state", &Query::default(), None)
            .map_err(|e| CoreError::Db(format!("read app_state: {e}")))?;
        let Some(row) = rows.first() else {
            return Ok(String::new());
        };
        match row.columns.get(&column_id) {
            Some(Value::Bytes(b)) => String::from_utf8(b.clone())
                .map_err(|e| CoreError::Db(format!("app_state.{key} not utf-8: {e}"))),
            Some(Value::Bool(b)) => Ok(b.to_string()),
            Some(Value::Int64(n)) => Ok(n.to_string()),
            Some(Value::Json(b)) => String::from_utf8(b.clone())
                .map_err(|e| CoreError::Db(format!("app_state.{key} not utf-8: {e}"))),
            _ => Ok(String::new()),
        }
    }

    /// Write `value` into the singleton `app_state` row for `key`. Preserves
    /// every other column by reading the current row first and re-`put`-ing
    /// the merged cells. Sync (mongreldb is sync) — callers from async
    /// contexts must wrap this in `spawn_blocking`.
    pub fn set_app_setting(&self, key: &str, value: &str) -> CoreResult<()> {
        let column_id = setting_column_for_key(key)?;
        let rows = self
            .inner
            .query_for_current_principal("app_state", &Query::default(), None)
            .map_err(|e| CoreError::Db(format!("read app_state: {e}")))?;
        let Some(row) = rows.first() else {
            return Err(CoreError::Db("app_state row missing".into()));
        };
        // Re-`put` the row with every column the schema defines. APP_RECENT
        // and APP_VAULT_PATH store bytes/json so they need to be cloned as-is;
        // unknown columns fall through as empty Bytes.
        let pk = row
            .columns
            .get(&col::APP_ID)
            .cloned()
            .unwrap_or(Value::Int64(1));
        let mut cells: Vec<(u16, Value)> = vec![
            (col::APP_ID, pk),
            (col::APP_SCHEMA_VER, Value::Int64(1)),
            (col::APP_VAULT_PATH, Value::Bytes(Vec::new())),
            (col::APP_LAST_OPENED, Value::Bytes(Vec::new())),
            (col::APP_RECENT, Value::Json(b"[]".to_vec())),
            (col::APP_TUTORIAL_DONE, Value::Bool(false)),
            (col::APP_THEME, Value::Bytes(b"dark".to_vec())),
            (col::APP_BETA, Value::Bool(false)),
            (col::APP_EMBED_QUANT, Value::Bytes(b"binary".to_vec())),
        ];
        // Preserve the current schema_version, vault_path, recent, tutorial_done,
        // beta, embedding_quant by overwriting the defaults from the existing row.
        for (col_id, val) in &row.columns {
            if let Some(slot) = cells.iter_mut().find(|(c, _)| *c == *col_id) {
                slot.1 = val.clone();
            }
        }
        // Now overwrite the targeted cell with a schema-compatible value.
        // Tauri transports settings as strings, but boolean app_state columns
        // must stay booleans in MongrelDB.
        let target_value = if column_id == col::APP_TUTORIAL_DONE || column_id == col::APP_BETA {
            match value {
                "true" => Value::Bool(true),
                "false" => Value::Bool(false),
                other => {
                    return Err(CoreError::Db(format!(
                        "app_state.{key} expects a boolean, got '{other}'"
                    )))
                }
            }
        } else {
            Value::Bytes(value.as_bytes().to_vec())
        };
        if let Some(slot) = cells.iter_mut().find(|(c, _)| *c == column_id) {
            slot.1 = target_value;
        }

        // `put` requires the PK column — the singleton row uses APP_ID = 1
        // so the same `Condition::Pk` lookup the read path uses maps 1:1 to
        // the row. Including it in `cells` lets mongreldb identify the row
        // to replace.
        self.inner
            .transaction_for_current_principal(|tx| tx.put("app_state", cells))
            .map_err(|e| CoreError::Db(format!("write app_state.{key}: {e}")))?;
        Ok(())
    }

    /// Record a fresh search query at the head of the `recent_searches` list.
    ///
    /// Semantics: empty / whitespace-only queries are a no-op (they would only
    /// clutter the palette's "Recent" group). Pre-existing duplicate queries
    /// are moved to the head rather than left in place — the brief asks for
    /// "dedup, prepend". The list is then truncated to
    /// [`RECENT_SEARCHES_CAP`] so the column can never grow without bound.
    pub fn push_recent_search(&self, query: &str) -> CoreResult<()> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(());
        }
        let existing = read_recent_searches(&self.inner)?;
        let mut updated: Vec<String> = existing.into_iter().filter(|q| q != trimmed).collect();
        updated.insert(0, trimmed.to_string());
        updated.truncate(RECENT_SEARCHES_CAP);
        write_app_state_cell(
            &self.inner,
            col::APP_RECENT,
            Value::Json(serde_json::to_vec(&updated)?),
        )
    }

    /// Record the id of the prompt the user most recently opened. Stored as
    /// a plain UTF-8 string in the `last_opened_prompt` column; the
    /// frontend reads it back on startup to pre-load the editor.
    pub fn set_last_opened(&self, prompt_id: &str) -> CoreResult<()> {
        write_app_state_cell(
            &self.inner,
            col::APP_LAST_OPENED,
            Value::Bytes(prompt_id.as_bytes().to_vec()),
        )
    }
}

/// Map the JS-facing setting key (as passed to the Tauri command) to its
/// app_state column id. Unknown keys return a typed error so the frontend
/// gets a deterministic string instead of a silent no-op.
fn setting_column_for_key(key: &str) -> CoreResult<u16> {
    match key {
        "tutorial_completed" => Ok(col::APP_TUTORIAL_DONE),
        "theme" => Ok(col::APP_THEME),
        "recent_searches" => Ok(col::APP_RECENT),
        "last_opened_prompt" => Ok(col::APP_LAST_OPENED),
        "embedding_quant" => Ok(col::APP_EMBED_QUANT),
        "beta_channel" => Ok(col::APP_BETA),
        other => Err(CoreError::Db(format!("unknown app_setting '{other}'"))),
    }
}

/// Read the singleton `app_state` row, returning every populated cell.
/// Centralises the "load row -> Vec of (col_id, Value)" pattern that the
/// `set_app_setting` / `push_recent_search` / `set_last_opened` writers all
/// share — without it each writer would have to re-query + re-decode the row.
fn read_app_state_row(db: &Database) -> CoreResult<Vec<(u16, Value)>> {
    let rows = db
        .query_for_current_principal("app_state", &Query::default(), None)
        .map_err(|e| CoreError::Db(format!("read app_state: {e}")))?;
    let Some(row) = rows.first() else {
        return Err(CoreError::Db("app_state row missing".into()));
    };
    Ok(row.columns.iter().map(|(k, v)| (*k, v.clone())).collect())
}

/// Re-`put` the singleton `app_state` row with `target` overwritten to
/// `value`. Other cells are preserved verbatim from the existing row; any
/// schema columns missing on the read side fall back to type-correct
/// defaults so the `put` always carries the full cell list.
fn write_app_state_cell(db: &Database, target: u16, value: Value) -> CoreResult<()> {
    let mut cells = read_app_state_row(db)?;
    let mut found = false;
    for (col_id, val) in cells.iter_mut() {
        if *col_id == target {
            *val = value.clone();
            found = true;
            break;
        }
    }
    if !found {
        cells.push((target, value));
    }
    db.transaction_for_current_principal(|tx| tx.put("app_state", cells))
        .map_err(|e| CoreError::Db(format!("write app_state cell {target}: {e}")))?;
    Ok(())
}

/// Parse the `recent_searches` column (a JSON array of strings) into a
/// `Vec<String>`. Treats missing/wrong-type cells as empty so callers can
/// read the column without branching on first-run vaults.
fn read_recent_searches(db: &Database) -> CoreResult<Vec<String>> {
    let cells = read_app_state_row(db)?;
    for (col_id, val) in cells {
        if col_id == col::APP_RECENT {
            if let Value::Json(b) = val {
                return serde_json::from_slice(&b).map_err(CoreError::from);
            }
            return Ok(Vec::new());
        }
    }
    Ok(Vec::new())
}

/// Maximum number of recent-search queries retained on disk. Older entries
/// are dropped first so the column never grows without bound.
const RECENT_SEARCHES_CAP: usize = 20;

#[cfg(test)]
mod tests {
    use super::*;
    use mongreldb_core::query::Query;
    use tempfile::TempDir;

    #[test]
    fn open_creates_db_and_tables() {
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();
        // 'prompts' table exists and is empty.
        let rows = db
            .handle()
            .query_for_current_principal("prompts", &Query::default(), None)
            .unwrap();
        assert_eq!(rows.len(), 0);
    }

    #[test]
    fn open_idempotent_on_existing_db() {
        // mongreldb-core enforces a process-wide single-instance lock per
        // database path, so a true "open twice in the same process" is not
        // supported. Instead we exercise the idempotent path of
        // `schema::create_all_tables` directly: invoking it against an
        // already-populated catalog must succeed without DDL errors.
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();
        crate::schema::create_all_tables(db.handle()).unwrap();
        // All six tables still present.
        let names = db.handle().table_names();
        for table in [
            "prompts",
            "app_state",
            "folders",
            "smart_folders",
            "embedding_queue",
            "plugins",
        ] {
            assert!(
                names.iter().any(|n| n == table),
                "table {table} missing after idempotent create_all_tables: {names:?}"
            );
        }
    }

    #[test]
    fn push_recent_search_dedupes_and_caps_at_twenty() {
        // Fresh vault: existing column is the seeded `[]`. After three pushes
        // — one of them a duplicate — the list must contain the dedup'd
        // entries in MRU order.
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();

        db.push_recent_search("first").unwrap();
        db.push_recent_search("second").unwrap();
        db.push_recent_search("first").unwrap(); // duplicate — moves to head

        let got = db.get_app_setting("recent_searches").unwrap();
        let parsed: Vec<String> = serde_json::from_str(&got).unwrap();
        assert_eq!(
            parsed,
            vec!["first".to_string(), "second".to_string()],
            "duplicate must move to head, not duplicate"
        );

        // Push 25 unique queries — only the most recent 20 must remain, in
        // reverse order (head = most recent).
        for i in 0..25 {
            db.push_recent_search(&format!("q{i}")).unwrap();
        }
        let got = db.get_app_setting("recent_searches").unwrap();
        let parsed: Vec<String> = serde_json::from_str(&got).unwrap();
        assert_eq!(parsed.len(), 20, "list must be capped at 20 entries");
        assert_eq!(parsed[0], "q24", "most recent must be at the head");
        assert_eq!(parsed[19], "q5", "older tail must be truncated");
    }

    #[test]
    fn push_recent_search_ignores_empty_and_whitespace() {
        // An empty/whitespace query would only clutter the palette's "Recent"
        // group — refuse to write it so the column stays clean.
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();

        db.push_recent_search("").unwrap();
        db.push_recent_search("   ").unwrap();
        db.push_recent_search("\t\n").unwrap();

        let got = db.get_app_setting("recent_searches").unwrap();
        assert_eq!(got, "[]", "empty / whitespace queries must not be recorded");
    }

    #[test]
    fn set_last_opened_round_trips_string() {
        // round-trip a 26-char ULID through the column to confirm the
        // Bytes(value) write + JSON/Bytes read path works. The frontend
        // would read this on app start to pre-load the editor.
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();
        let pid = "01HXXXXXXXXXXXXXXXXXXXXXX";

        db.set_last_opened(pid).unwrap();
        let got = db.get_app_setting("last_opened_prompt").unwrap();
        assert_eq!(got, pid);

        // Overwrite with a new id — must replace, not append.
        db.set_last_opened("01HYYYYYYYYYYYYYYYYYYYYYY").unwrap();
        let got = db.get_app_setting("last_opened_prompt").unwrap();
        assert_eq!(got, "01HYYYYYYYYYYYYYYYYYYYYYY");
    }

    #[test]
    fn tutorial_completion_round_trips_as_boolean_setting() {
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();

        assert_eq!(db.get_app_setting("tutorial_completed").unwrap(), "false");
        db.set_app_setting("tutorial_completed", "true").unwrap();
        assert_eq!(db.get_app_setting("tutorial_completed").unwrap(), "true");

        let rows = db
            .handle()
            .query_for_current_principal("app_state", &Query::default(), None)
            .unwrap();
        assert!(matches!(
            rows[0].columns.get(&col::APP_TUTORIAL_DONE),
            Some(Value::Bool(true))
        ));
    }

    #[test]
    fn beta_channel_round_trips_as_boolean_setting() {
        // M7.2: `beta_channel` is the opt-in for pre-release auto-updates.
        // The boolean coercion in `set_app_setting` must coerce the wire
        // "true"/"false" string into a `Value::Bool` so the column stays
        // type-correct (the schema declares `APP_BETA` as `TypeId::Bool`).
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();

        // Seeded default is `false`.
        assert_eq!(db.get_app_setting("beta_channel").unwrap(), "false");

        db.set_app_setting("beta_channel", "true").unwrap();
        assert_eq!(db.get_app_setting("beta_channel").unwrap(), "true");

        let rows = db
            .handle()
            .query_for_current_principal("app_state", &Query::default(), None)
            .unwrap();
        assert!(
            matches!(rows[0].columns.get(&col::APP_BETA), Some(Value::Bool(true))),
            "APP_BETA must be stored as Value::Bool, not Value::Bytes"
        );

        // Toggle back to false and re-read.
        db.set_app_setting("beta_channel", "false").unwrap();
        assert_eq!(db.get_app_setting("beta_channel").unwrap(), "false");
    }

    #[test]
    fn beta_channel_rejects_non_boolean_wire_value() {
        // Defence-in-depth: the same wire-value guard `tutorial_completed`
        // has, applied to the new boolean column. Refuses anything other
        // than "true" / "false" so a malformed frontend payload can't
        // corrupt the column type.
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();

        let err = db
            .set_app_setting("beta_channel", "yes")
            .expect_err("non-boolean wire value must be rejected");
        assert!(
            err.to_string().contains("beta_channel expects a boolean"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn write_app_state_cell_preserves_neighbouring_columns() {
        // Setting one column must not trample the surrounding cells — in
        // particular the seeded `embedding_quant` defaults and any theme
        // the user previously wrote. Regression guard for the helper
        // introduced alongside `push_recent_search`.
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();
        db.set_app_setting("theme", "light").unwrap();
        db.push_recent_search("hello").unwrap();
        db.set_last_opened("id-1").unwrap();

        assert_eq!(
            db.get_app_setting("theme").unwrap(),
            "light",
            "theme must survive"
        );
        assert_eq!(
            db.get_app_setting("recent_searches").unwrap(),
            r#"["hello"]"#
        );
        assert_eq!(db.get_app_setting("last_opened_prompt").unwrap(), "id-1");
    }
}
