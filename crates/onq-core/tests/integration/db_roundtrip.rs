//! Conformance test — schema + indexes work end-to-end.
//!
//! Opens a fresh vault (which seeds every table from
//! [`onq_core::schema`]), inserts one row into `prompts` covering the
//! most-exercised column kinds (Bytes, Bool, Int64, Json), and verifies the
//! row round-trips through the typed query API. Proves the schema from M2.2
//! (tables created, columns typed, queries return data) actually works.
//!
//! Adapted from the original Task 2.5 brief: the brief pre-dated the
//! async→sync correction and the typed-API switch, so this test uses
//! `Db::open(path, &str)` (sync, takes a passphrase) and writes via the
//! typed `transaction_for_current_principal | put` path rather than a
//! `db.execute(SQL)` call. The point of the test is unchanged: prove the
//! schema works.

use mongreldb_core::{Query, Value};
use onq_core::db::Db;
use onq_core::schema::col;
use tempfile::TempDir;

#[test]
fn schema_creates_and_indexes_queryable() {
    let dir = TempDir::new().unwrap();
    let db = Db::open(dir.path(), "test-passphrase").unwrap();

    // Insert one row covering Bytes / Bool / Int64 / Json / Embedding columns.
    // `embedding` is NOT NULL and has no default, so we supply a 384-component
    // zero vector — the conformant path here is "schema accepts typed writes";
    // real embeddings are produced asynchronously by the M3 search worker.
    let embedding = vec![0.0f32; 384];
    db.handle()
        .transaction_for_current_principal(|tx| {
            tx.put(
                "prompts",
                vec![
                    (col::PROMPTS_ID, Value::Bytes(b"01H".to_vec())),
                    (col::PROMPTS_TITLE, Value::Bytes(b"Hello".to_vec())),
                    (col::PROMPTS_FOLDER, Value::Bytes(b"inbox".to_vec())),
                    (col::PROMPTS_BODY, Value::Bytes(b"World".to_vec())),
                    (col::PROMPTS_TAGS, Value::Json(br#"["a"]"#.to_vec())),
                    (col::PROMPTS_FAVORITE, Value::Bool(false)),
                    (col::PROMPTS_LOCKED, Value::Bool(false)),
                    (col::PROMPTS_CHAR, Value::Int64(5)),
                    (col::PROMPTS_CREATED, Value::Int64(0)),
                    (col::PROMPTS_UPDATED, Value::Int64(0)),
                    (col::PROMPTS_EMBED, Value::Embedding(embedding)),
                ],
            )?;
            Ok(())
        })
        .unwrap();

    // No-condition query = all rows visible to the current principal.
    let rows = db
        .handle()
        .query_for_current_principal("prompts", &Query::default(), None)
        .unwrap();

    assert_eq!(rows.len(), 1, "expected exactly one round-tripped row");
    let row = &rows[0];
    assert_eq!(
        row.columns.get(&col::PROMPTS_ID),
        Some(&Value::Bytes(b"01H".to_vec())),
        "id column round-trip"
    );
    assert_eq!(
        row.columns.get(&col::PROMPTS_TITLE),
        Some(&Value::Bytes(b"Hello".to_vec())),
        "title column round-trip"
    );
    assert_eq!(
        row.columns.get(&col::PROMPTS_BODY),
        Some(&Value::Bytes(b"World".to_vec())),
        "body column round-trip"
    );
    assert_eq!(
        row.columns.get(&col::PROMPTS_CHAR),
        Some(&Value::Int64(5)),
        "char_count column round-trip"
    );
    assert_eq!(
        row.columns.get(&col::PROMPTS_TAGS),
        Some(&Value::Json(br#"["a"]"#.to_vec())),
        "tags column round-trip"
    );
}
