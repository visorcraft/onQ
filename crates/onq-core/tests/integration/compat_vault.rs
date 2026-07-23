//! Cross-version golden vault: on-disk bytes from a known good engine must
//! remain openable, readable, and writable after MongrelDB / onQ upgrades.
//!
//! Synthetic migration unit tests build "old" schemas with the *current*
//! `mongreldb-core`. This suite instead materializes a committed encrypted
//! vault snapshot (see `tests/fixtures/compat-vault.tar.gz`) and exercises
//! the real open path — the regression that would ship a silent format break.
//!
//! Regenerate the fixture after intentional schema/engine changes:
//!
//! ```bash
//! ./scripts/generate-compat-vault.sh
//! ```

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use mongreldb_core::query::{Condition, Query};
use mongreldb_core::Value;
use onq_core::db::Db;
use onq_core::schema::col;
use tar::{Archive, Builder, Header};
use tempfile::TempDir;

/// Passphrase baked into the committed fixture. Not a secret — test only.
const PASSPHRASE: &str = "compat-fixture-passphrase";

/// Stable primary key of the single seed prompt.
const PROMPT_ID: &[u8] = b"01COMPATFIXTURE0000000001";

const PROMPT_TITLE: &[u8] = b"Compat fixture prompt";
const PROMPT_BODY: &[u8] = b"Stable body for cross-version open tests.";
const PROMPT_TAGS: &[u8] = br#"["compat","golden"]"#;
const PROMPT_CHAR: i64 = 42;
const PROMPT_TS: i64 = 1_700_000_000_000;

/// Expected `app_state.schema_version` after open + migrator (latest onQ).
const LATEST_SCHEMA_VERSION: i64 = 3;

fn fixture_tarball_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/compat-vault.tar.gz")
}

fn fixture_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/compat-vault.md")
}

/// Unpack the committed tarball into a fresh temp vault root and return it.
fn materialize_fixture() -> TempDir {
    let tar_path = fixture_tarball_path();
    assert!(
        tar_path.is_file(),
        "missing golden vault fixture at {}; run ./scripts/generate-compat-vault.sh",
        tar_path.display()
    );
    let bytes = fs::read(&tar_path).expect("read fixture tarball");
    let dest = TempDir::new().expect("tempdir");
    unpack_tree(&bytes, dest.path()).expect("unpack fixture");
    // Sanity: search-index must exist (Db::open creates otherwise — that would
    // silently defeat the whole point of a golden-byte test).
    let index = dest.path().join(".onq").join("search-index");
    assert!(
        index.is_dir(),
        "fixture missing .onq/search-index after unpack"
    );
    dest
}

fn unpack_tree(payload: &[u8], dest: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    let decoder = GzDecoder::new(payload);
    let mut archive = Archive::new(decoder);
    archive.unpack(dest)?;
    Ok(())
}

fn pack_tree(vault_path: &Path) -> std::io::Result<Vec<u8>> {
    let mut raw = Vec::new();
    {
        let enc = GzEncoder::new(&mut raw, Compression::default());
        let mut builder = Builder::new(enc);
        append_dir(&mut builder, vault_path, Path::new(""))?;
        let enc = builder.into_inner()?;
        enc.finish()?;
    }
    Ok(raw)
}

fn append_dir<W: Write>(builder: &mut Builder<W>, abs: &Path, rel: &Path) -> std::io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(abs)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let name = entry.file_name();
        let child_abs = entry.path();
        let child_rel = rel.join(&name);
        let meta = entry.metadata()?;
        let rel_str = child_rel.to_string_lossy().replace('\\', "/");
        if meta.is_dir() {
            let mut header = Header::new_gnu();
            header.set_entry_type(tar::EntryType::Directory);
            header.set_mode(0o755);
            header.set_size(0);
            header.set_cksum();
            builder.append_data(&mut header, format!("{rel_str}/"), std::io::empty())?;
            append_dir(builder, &child_abs, &child_rel)?;
        } else if meta.is_file() {
            let mut file = fs::File::open(&child_abs)?;
            let mut header = Header::new_gnu();
            header.set_entry_type(tar::EntryType::Regular);
            header.set_mode(0o644);
            header.set_size(meta.len());
            header.set_cksum();
            builder.append_data(&mut header, Path::new(rel_str.as_str()), &mut file)?;
        }
    }
    Ok(())
}

fn seed_prompt(db: &Db) {
    let embedding = vec![0.0f32; 384];
    db.handle()
        .transaction_for_current_principal(|tx| {
            tx.put(
                "prompts",
                vec![
                    (col::PROMPTS_ID, Value::Bytes(PROMPT_ID.to_vec())),
                    (col::PROMPTS_TITLE, Value::Bytes(PROMPT_TITLE.to_vec())),
                    (col::PROMPTS_FOLDER, Value::Bytes(b"inbox".to_vec())),
                    (col::PROMPTS_BODY, Value::Bytes(PROMPT_BODY.to_vec())),
                    (col::PROMPTS_TAGS, Value::Json(PROMPT_TAGS.to_vec())),
                    (col::PROMPTS_FAVORITE, Value::Bool(true)),
                    (col::PROMPTS_LOCKED, Value::Bool(false)),
                    (col::PROMPTS_CHAR, Value::Int64(PROMPT_CHAR)),
                    (col::PROMPTS_CREATED, Value::Int64(PROMPT_TS)),
                    (col::PROMPTS_UPDATED, Value::Int64(PROMPT_TS)),
                    (col::PROMPTS_EMBED, Value::Embedding(embedding)),
                ],
            )?;
            Ok(())
        })
        .expect("seed prompt");
}

fn assert_tables_present(db: &Db) {
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
            "expected table {table} after opening golden vault; got {names:?}"
        );
    }
}

fn assert_seed_prompt(db: &Db) {
    let q = Query {
        conditions: vec![Condition::Pk(PROMPT_ID.to_vec())],
        ..Query::default()
    };
    let rows = db
        .handle()
        .query_for_current_principal("prompts", &q, None)
        .expect("query seed prompt");
    assert_eq!(rows.len(), 1, "golden prompt missing after open");
    let row = &rows[0];
    assert_eq!(
        row.columns.get(&col::PROMPTS_TITLE),
        Some(&Value::Bytes(PROMPT_TITLE.to_vec()))
    );
    assert_eq!(
        row.columns.get(&col::PROMPTS_BODY),
        Some(&Value::Bytes(PROMPT_BODY.to_vec()))
    );
    assert_eq!(
        row.columns.get(&col::PROMPTS_TAGS),
        Some(&Value::Json(PROMPT_TAGS.to_vec()))
    );
    assert_eq!(
        row.columns.get(&col::PROMPTS_FAVORITE),
        Some(&Value::Bool(true))
    );
    assert_eq!(
        row.columns.get(&col::PROMPTS_CHAR),
        Some(&Value::Int64(PROMPT_CHAR))
    );
}

fn assert_app_state(db: &Db) {
    assert_eq!(
        db.get_app_setting("theme").expect("theme"),
        "light",
        "seeded theme must survive open/migrate"
    );
    assert_eq!(
        db.get_app_setting("embedding_quant").expect("quant"),
        "binary"
    );
    // schema_version is not a public app_setting key; read the column directly.
    let rows = db
        .handle()
        .query_for_current_principal("app_state", &Query::default(), None)
        .expect("read app_state");
    let row = rows.first().expect("app_state row");
    match row.columns.get(&col::APP_SCHEMA_VER) {
        Some(Value::Int64(v)) => assert_eq!(
            *v, LATEST_SCHEMA_VERSION,
            "migrator must leave vault at latest schema_version"
        ),
        other => panic!("schema_version missing/wrong type: {other:?}"),
    }
}

/// Primary gate: committed on-disk bytes open, migrate, read, write, re-open.
#[test]
fn golden_vault_remains_usable_across_engine() {
    let vault = materialize_fixture();

    let db = Db::open(vault.path(), PASSPHRASE)
        .unwrap_or_else(|e| panic!("open golden vault failed (format regression?): {e}"));
    assert_tables_present(&db);
    assert_seed_prompt(&db);
    assert_app_state(&db);

    // Write must succeed and round-trip through a second open (writable path).
    db.set_app_setting("theme", "dark")
        .expect("write app_state after open");
    drop(db);

    let db2 = Db::open(vault.path(), PASSPHRASE).expect("re-open after write");
    assert_eq!(db2.get_app_setting("theme").unwrap(), "dark");
    assert_seed_prompt(&db2);
}

/// Wrong passphrase must fail closed — guards against "test accidentally
/// created a fresh vault because the open path fell through".
#[test]
fn golden_vault_rejects_wrong_passphrase() {
    let vault = materialize_fixture();
    let err = match Db::open(vault.path(), "definitely-not-the-passphrase") {
        Ok(_) => panic!("wrong passphrase must not open golden vault"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(
        msg.contains("open") || msg.to_lowercase().contains("decrypt") || msg.contains("cipher"),
        "unexpected error for bad passphrase: {msg}"
    );
    // Index must still be the original fixture (not replaced by create path).
    assert!(vault.path().join(".onq").join("search-index").is_dir());
}

/// Rewrite `tests/fixtures/compat-vault.tar.gz` from the current engine.
///
/// ```bash
/// ONQ_REGEN_COMPAT_FIXTURE=1 cargo test -p onq-core --test compat_vault \
///   regenerate_compat_fixture -- --ignored --nocapture
/// ```
#[test]
#[ignore = "run via scripts/generate-compat-vault.sh to rewrite committed fixture"]
fn regenerate_compat_fixture() {
    assert_eq!(
        std::env::var("ONQ_REGEN_COMPAT_FIXTURE").ok().as_deref(),
        Some("1"),
        "set ONQ_REGEN_COMPAT_FIXTURE=1 to confirm intentional fixture rewrite"
    );

    let dir = TempDir::new().expect("tempdir");
    let vault = dir.path();
    let db = Db::open(vault, PASSPHRASE).expect("create vault for fixture");
    db.set_app_setting("theme", "light").expect("set theme");
    db.set_app_setting("embedding_quant", "binary")
        .expect("set quant");
    seed_prompt(&db);
    // Sanity before packing.
    assert_tables_present(&db);
    assert_seed_prompt(&db);
    drop(db);

    let tarball = pack_tree(vault).expect("pack vault tree");
    let out = fixture_tarball_path();
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).expect("fixtures dir");
    }
    fs::write(&out, &tarball).expect("write tarball");

    let mongreldb = std::env::var("ONQ_COMPAT_MONGRELDB_VERSION")
        .unwrap_or_else(|_| "see Cargo.lock mongreldb-core".into());
    let manifest = format!(
        r#"# Compat vault golden fixture

Encrypted onQ vault snapshot used by `compat_vault` integration tests.

| Field | Value |
|-------|-------|
| Passphrase | `{PASSPHRASE}` |
| Prompt id | `{id}` |
| Prompt title | `{title}` |
| Theme | `light` |
| Favorite | `true` |
| `schema_version` (at generation) | `{LATEST_SCHEMA_VERSION}` |
| mongreldb-core (at generation) | `{mongreldb}` |

## What this catches

Opening this tarball exercises the real MongrelDB on-disk format plus onQ
migrations. Synthetic unit tests that `create_encrypted` with the *current*
engine cannot catch format breaks; this fixture can.

## Regenerate

After an intentional schema or engine change that invalidates old bytes:

```bash
./scripts/generate-compat-vault.sh
```

Commit the updated `compat-vault.tar.gz` and this file together.
"#,
        id = String::from_utf8_lossy(PROMPT_ID),
        title = String::from_utf8_lossy(PROMPT_TITLE),
    );
    fs::write(fixture_manifest_path(), manifest).expect("write manifest");

    eprintln!(
        "wrote {} ({} bytes) and {}",
        out.display(),
        tarball.len(),
        fixture_manifest_path().display()
    );
}
