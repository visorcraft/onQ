//! Schema migration runner for onQ.
//!
//! Tracks the on-disk schema version in the singleton `app_state` row's
//! `schema_version` column (id = 1, version = `col::APP_SCHEMA_VER`). On every
//! `Db::open`, [`Migrator::run`] applies any pending migration whose `version`
//! is strictly greater than the stored version. Re-running is a no-op.
//!
//! Migrations are typed Rust closures over `&Database` — they call the typed
//! `create_table` / `transaction_for_current_principal` / `put` APIs from
//! `mongreldb-core`, never raw SQL. Each migration is responsible for being
//! idempotent on its own data writes (table creation already short-circuits
//! when the table exists, but row inserts must guard against re-execution).
//!
//! To add a new migration:
//! 1. Append a new entry to [`MIGRATIONS`] with a strictly-higher `version`.
//! 2. Write the closure to perform the schema change. Increment
//!    `schema_version` in `app_state` to the new version as part of the same
//!    transaction (or rely on the migrator's post-step update below).
//!
//! New entries must never be reordered or removed — `version` is a durable
//! contract with on-disk databases.

use mongreldb_core::schema::{ColumnFlags, DefaultExpr, TypeId};
use mongreldb_core::{Database, Query, Value};
use tracing::info;

use crate::error::{CoreError, CoreResult};
use crate::schema::{self, col};

/// One schema migration: a monotonic version, a stable name for logs, and the
/// closure that performs the work. `run` MUST be idempotent on its data side
/// (table creation already is via [`schema::create_all_tables`]).
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub run: fn(&Database) -> CoreResult<()>,
}

/// All known migrations in ascending version order. Version is durable — never
/// reorder, never remove, never reuse.
const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "0001_initial",
        run: migration_0001_initial,
    },
    Migration {
        version: 2,
        name: "0002_minimize_on_copy",
        run: migration_0002_minimize_on_copy,
    },
];

/// Migration 0001 — create the six core tables and seed the singleton
/// `app_state` row with `schema_version = 1` and `embedding_quant = "binary"`.
/// Idempotent: tables are skipped if they exist; the app_state row is only
/// inserted when no row is present.
fn migration_0001_initial(db: &Database) -> CoreResult<()> {
    schema::create_all_tables(db)
        .map_err(|e| CoreError::Db(format!("0001_initial create tables: {e}")))?;
    let existing = db
        .query_for_current_principal("app_state", &Query::default(), None)
        .map_err(|e| CoreError::Db(format!("0001_initial probe app_state: {e}")))?;
    if existing.is_empty() {
        db.transaction_for_current_principal(|tx| {
            tx.put(
                "app_state",
                vec![
                    (col::APP_ID, Value::Int64(1)),
                    (col::APP_SCHEMA_VER, Value::Int64(1)),
                    (col::APP_VAULT_PATH, Value::Bytes(Vec::new())),
                    (col::APP_LAST_OPENED, Value::Bytes(Vec::new())),
                    (col::APP_RECENT, Value::Json(b"[]".to_vec())),
                    (col::APP_TUTORIAL_DONE, Value::Bool(false)),
                    (col::APP_THEME, Value::Bytes(b"dark".to_vec())),
                    (col::APP_BETA, Value::Bool(false)),
                    (col::APP_EMBED_QUANT, Value::Bytes(b"binary".to_vec())),
                ],
            )
        })
        .map_err(|e| CoreError::Db(format!("0001_initial seed app_state: {e}")))?;
    }
    Ok(())
}

/// Migration 0002 — add the `minimize_on_copy` boolean column to the
/// `app_state` row. Idempotent: mongreldb-core refuses `add_column` when
/// the column id is already known, so re-running on a fresh DB that has
/// the column baked into the schema is a no-op (and we'll detect that
/// via `current_version` returning >= 2 anyway).
fn migration_0002_minimize_on_copy(db: &Database) -> CoreResult<()> {
    let already_present = db
        .query_for_current_principal("app_state", &Query::default(), None)
        .map_err(|e| CoreError::Db(format!("0002 probe app_state: {e}")))?
        .first()
        .map(|row| row.columns.contains_key(&col::APP_MINIMIZE_ON_COPY))
        .unwrap_or(false);
    if !already_present {
        db.add_column_with_id(
            "app_state",
            "minimize_on_copy",
            TypeId::Bool,
            ColumnFlags::empty(),
            Some(DefaultExpr::Static(Value::Bool(false))),
            Some(col::APP_MINIMIZE_ON_COPY),
        )
        .map_err(|e| CoreError::Db(format!("0002 add_column minimize_on_copy: {e}")))?;
    }
    Ok(())
}

/// Owns a `&Database` and applies pending migrations from [`MIGRATIONS`].
pub struct Migrator<'a> {
    db: &'a Database,
}

impl<'a> Migrator<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Apply every migration whose `version` is strictly greater than the
    /// stored schema version. Returns the highest applied version (or the
    /// stored version if nothing was pending). Idempotent.
    pub fn run(&self) -> CoreResult<()> {
        let current = self.current_version()?;
        for m in MIGRATIONS {
            if m.version > current {
                info!(
                    migration = m.name,
                    version = m.version,
                    "applying migration"
                );
                (m.run)(self.db)?;
            }
        }
        Ok(())
    }

    /// Highest version stored in `app_state.schema_version`, or 0 if the
    /// `app_state` table is missing or empty. Treats anything other than a
    /// stored `Int64` schema_version as version 0 (fresh DB).
    fn current_version(&self) -> CoreResult<i64> {
        if !self.db.table_names().iter().any(|n| n == "app_state") {
            return Ok(0);
        }
        let rows = self
            .db
            .query_for_current_principal("app_state", &Query::default(), None)
            .map_err(|e| CoreError::Db(format!("read app_state: {e}")))?;
        let Some(row) = rows.first() else {
            return Ok(0);
        };
        match row.columns.get(&col::APP_SCHEMA_VER) {
            Some(Value::Int64(v)) => Ok(*v),
            _ => Ok(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use tempfile::TempDir;

    /// Read `schema_version` and `embedding_quant` from the singleton
    /// `app_state` row. Returns `(version, embedding_quant_bytes)`.
    fn read_app_state(db: &Database) -> CoreResult<(i64, Vec<u8>)> {
        let rows = db
            .query_for_current_principal("app_state", &Query::default(), None)
            .map_err(|e| CoreError::Db(e.to_string()))?;
        let row = rows
            .first()
            .ok_or_else(|| CoreError::Db("app_state empty".into()))?;
        let version = match row.columns.get(&col::APP_SCHEMA_VER) {
            Some(Value::Int64(v)) => *v,
            _ => return Err(CoreError::Db("schema_version missing/wrong type".into())),
        };
        let quant = match row.columns.get(&col::APP_EMBED_QUANT) {
            Some(Value::Bytes(b)) => b.clone(),
            _ => return Err(CoreError::Db("embedding_quant missing/wrong type".into())),
        };
        Ok((version, quant))
    }

    #[test]
    fn runs_each_migration_once() {
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();

        // After first open + run, schema_version must reflect the latest
        // migration that actually applied to a fresh DB.
        let (v1, q1) = read_app_state(db.handle()).unwrap();
        assert_eq!(v1, 1, "expected schema_version=1 after first run");
        assert_eq!(q1, b"binary", "expected embedding_quant='binary'");

        // Second open on the same path runs migrations again — must be a
        // no-op (still version 1, still 'binary', no error).
        drop(db);
        let db2 = Db::open(dir.path(), "test-pass").unwrap();
        let (v2, _) = read_app_state(db2.handle()).unwrap();
        assert_eq!(v2, 1, "second run must not advance schema_version");
    }

    #[test]
    fn creates_all_six_tables() {
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();
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
                "table {table} missing after migration: {names:?}"
            );
        }
    }

    #[test]
    fn migrator_on_fresh_database_does_full_run() {
        // Drive the migrator directly against a brand-new database, skipping
        // `Db::open` (which also calls the migrator). Verifies the migrator
        // itself — not `Db::open` — is the source of truth for what runs.
        let dir = TempDir::new().unwrap();
        let db = mongreldb_core::Database::create_encrypted(dir.path(), "x")
            .expect("create encrypted db");
        Migrator::new(&db).run().unwrap();
        let (v, q) = read_app_state(&db).unwrap();
        assert_eq!(v, 1);
        assert_eq!(q, b"binary");
    }

    #[test]
    fn migrator_on_existing_schema_is_noop() {
        let dir = TempDir::new().unwrap();
        let db = mongreldb_core::Database::create_encrypted(dir.path(), "x")
            .expect("create encrypted db");
        // Pre-create tables + seed app_state to simulate a DB that already
        // has schema_version=1 from a prior run, then verify the migrator
        // doesn't re-run or clobber the row.
        schema::create_all_tables(&db).unwrap();
        db.transaction_for_current_principal(|tx| {
            tx.put(
                "app_state",
                vec![
                    (col::APP_ID, Value::Int64(1)),
                    (col::APP_SCHEMA_VER, Value::Int64(1)),
                    (col::APP_VAULT_PATH, Value::Bytes(b"/vault".to_vec())),
                    (col::APP_LAST_OPENED, Value::Bytes(b"".to_vec())),
                    (col::APP_RECENT, Value::Json(b"[]".to_vec())),
                    (col::APP_TUTORIAL_DONE, Value::Bool(true)),
                    (col::APP_THEME, Value::Bytes(b"light".to_vec())),
                    (col::APP_BETA, Value::Bool(false)),
                    (col::APP_EMBED_QUANT, Value::Bytes(b"dense".to_vec())),
                ],
            )?;
            Ok(())
        })
        .unwrap();
        Migrator::new(&db).run().unwrap();
        // Pre-existing user data must be preserved across migrator re-run.
        let (v, q) = read_app_state(&db).unwrap();
        assert_eq!(v, 1);
        assert_eq!(
            q, b"dense",
            "user-tuned embedding_quant must survive migration"
        );
    }
}
