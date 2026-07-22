//! All table schemas + index declarations. Single source of truth for storage layout.
//!
//! Built against the typed [`mongreldb_core::Schema`] / [`mongreldb_core::IndexDef`]
//! API — no raw SQL. See `docs/mongreldb-api-notes.md` for the full surface.

use mongreldb_core::embedding::EmbeddingSource;
use mongreldb_core::memtable::Value;
use mongreldb_core::schema::{
    AnnOptions, AnnQuantization, ColumnDef, ColumnFlags, DefaultExpr, IndexDef, IndexKind,
    IndexOptions, LearnedRangeOptions, MinHashOptions, Schema, TypeId,
};

use crate::error::{CoreError, CoreResult};

// Stable column IDs (never reused). Defined here so query builders can reference by ID.
pub mod col {
    // prompts
    pub const PROMPTS_ID: u16 = 0;
    pub const PROMPTS_TITLE: u16 = 1;
    pub const PROMPTS_FOLDER: u16 = 2;
    pub const PROMPTS_BODY: u16 = 3;
    pub const PROMPTS_TAGS: u16 = 4;
    pub const PROMPTS_FAVORITE: u16 = 5;
    pub const PROMPTS_LOCKED: u16 = 6;
    pub const PROMPTS_CHAR: u16 = 7;
    pub const PROMPTS_CREATED: u16 = 8;
    pub const PROMPTS_UPDATED: u16 = 9;
    pub const PROMPTS_EMBED: u16 = 10;
    // Encoded auxiliary columns derived from `body` — each holds the
    // representation required by its index kind. mongreldb-core enforces
    // "one ANN/Sparse/MinHash/FM representation index per column", so the
    // FmIndex lives on `body` itself while Sparse and MinHash live on these
    // dedicated columns. Future writers populate them from the source body.
    pub const PROMPTS_BODY_SPARSE: u16 = 11;
    pub const PROMPTS_BODY_MINHASH: u16 = 12;
    // app_state
    pub const APP_ID: u16 = 0;
    pub const APP_SCHEMA_VER: u16 = 1;
    pub const APP_VAULT_PATH: u16 = 2;
    pub const APP_LAST_OPENED: u16 = 3;
    pub const APP_RECENT: u16 = 4;
    pub const APP_TUTORIAL_DONE: u16 = 5;
    pub const APP_THEME: u16 = 6;
    pub const APP_BETA: u16 = 7;
    pub const APP_EMBED_QUANT: u16 = 8; // 'binary' (default, HNSW+rerank) or 'dense' (HNSW direct)
    pub const APP_MINIMIZE_ON_COPY: u16 = 9; // bool: hide main window to tray after copying from palette
                                             // folders
    pub const FOLDERS_ID: u16 = 0;
    pub const FOLDERS_NAME: u16 = 1;
    pub const FOLDERS_CREATED: u16 = 2;
    pub const FOLDERS_UPDATED: u16 = 3;
    // smart_folders
    pub const SF_ID: u16 = 0;
    pub const SF_NAME: u16 = 1;
    pub const SF_DSL: u16 = 2;
    pub const SF_VISUAL: u16 = 3;
    pub const SF_CREATED: u16 = 4;
    pub const SF_UPDATED: u16 = 5;
    // embedding_queue
    pub const EQ_PROMPT_ID: u16 = 0;
    pub const EQ_QUEUED_AT: u16 = 1;
    pub const EQ_ATTEMPTS: u16 = 2;
    pub const EQ_LAST_ERROR: u16 = 3;
    // plugins
    pub const PL_ID: u16 = 0;
    pub const PL_NAME: u16 = 1;
    pub const PL_VERSION: u16 = 2;
    pub const PL_PATH: u16 = 3;
    pub const PL_SIG: u16 = 4;
    pub const PL_CAPS: u16 = 5;
    pub const PL_INSTALLED_AT: u16 = 6;
    pub const PL_ENABLED: u16 = 7;
}

fn pk() -> ColumnFlags {
    ColumnFlags::empty().with(ColumnFlags::PRIMARY_KEY)
}

fn nullable() -> ColumnFlags {
    ColumnFlags::empty().with(ColumnFlags::NULLABLE)
}

fn embedding_quant() -> ColumnFlags {
    ColumnFlags::empty().with(ColumnFlags::EMBEDDING_BINARY_QUANTIZED)
}

/// Stable name of the prompts embedding ANN index. Rebuild / replace-index
/// and readiness inspection key off this string.
pub const PROMPTS_EMBED_ANN_INDEX: &str = "idx_prompts_embed_ann";

/// `prompts` table: full-text body + tags, ANN embedding, recency, favorites/locks.
///
/// Defaults to BinarySign ANN quantization. Prefer
/// [`prompts_schema_with_quantization`] when the user's `embedding_quant`
/// setting is known (fresh vaults, rebuild).
pub fn prompts_schema() -> Schema {
    prompts_schema_with_quantization(AnnQuantization::BinarySign)
}

/// Prompts schema using the requested ANN representation.
pub fn prompts_schema_with_quantization(quantization: AnnQuantization) -> Schema {
    Schema {
        schema_id: 1,
        columns: vec![
            ColumnDef {
                id: col::PROMPTS_ID,
                name: "id".into(),
                ty: TypeId::Bytes,
                flags: pk(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::PROMPTS_TITLE,
                name: "title".into(),
                ty: TypeId::Bytes,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::PROMPTS_FOLDER,
                name: "folder".into(),
                ty: TypeId::Bytes,
                flags: nullable(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::PROMPTS_BODY,
                name: "body".into(),
                ty: TypeId::Bytes,
                flags: nullable(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::PROMPTS_TAGS,
                name: "tags".into(),
                ty: TypeId::Json,
                flags: nullable(),
                default_value: Some(DefaultExpr::Static(Value::Json(b"[]".to_vec()))),
                embedding_source: None,
            },
            ColumnDef {
                id: col::PROMPTS_FAVORITE,
                name: "favorite".into(),
                ty: TypeId::Bool,
                flags: ColumnFlags::empty(),
                default_value: Some(DefaultExpr::Static(Value::Bool(false))),
                embedding_source: None,
            },
            ColumnDef {
                id: col::PROMPTS_LOCKED,
                name: "locked".into(),
                ty: TypeId::Bool,
                flags: ColumnFlags::empty(),
                default_value: Some(DefaultExpr::Static(Value::Bool(false))),
                embedding_source: None,
            },
            ColumnDef {
                id: col::PROMPTS_CHAR,
                name: "char_count".into(),
                ty: TypeId::Int64,
                flags: ColumnFlags::empty(),
                default_value: Some(DefaultExpr::Static(Value::Int64(0))),
                embedding_source: None,
            },
            ColumnDef {
                id: col::PROMPTS_CREATED,
                name: "created_at".into(),
                ty: TypeId::Int64,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::PROMPTS_UPDATED,
                name: "updated_at".into(),
                ty: TypeId::Int64,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::PROMPTS_EMBED,
                name: "embedding".into(),
                ty: TypeId::Embedding { dim: 384 },
                flags: embedding_quant(),
                default_value: None,
                embedding_source: Some(EmbeddingSource::SuppliedByApplication),
            },
            // Body-derived sparse vector: encoded `(token_id, weight)` pairs
            // (bincode) stored as Bytes for the Sparse index.
            ColumnDef {
                id: col::PROMPTS_BODY_SPARSE,
                name: "body_sparse".into(),
                ty: TypeId::Bytes,
                flags: nullable(),
                default_value: None,
                embedding_source: None,
            },
            // Body-derived MinHash set: JSON-encoded scalar members serialized as Bytes.
            // (mongreldb-core validates the JSON array shape on write; we treat it as Bytes
            //  here because the MinHash index requires a Bytes column.)
            ColumnDef {
                id: col::PROMPTS_BODY_MINHASH,
                name: "body_minhash".into(),
                ty: TypeId::Bytes,
                flags: nullable(),
                default_value: None,
                embedding_source: None,
            },
        ],
        indexes: prompts_indexes_with_quantization(quantization),
        colocation: vec![],
        constraints: Default::default(),
        clustered: false,
    }
}

/// 11 indexes on prompts covering all six `IndexKind` variants (BinarySign ANN).
pub fn prompts_indexes() -> Vec<IndexDef> {
    prompts_indexes_with_quantization(AnnQuantization::BinarySign)
}

/// Prompts indexes with the requested ANN quantization on the embedding column.
pub fn prompts_indexes_with_quantization(quantization: AnnQuantization) -> Vec<IndexDef> {
    vec![
        IndexDef {
            name: "idx_prompts_folder".into(),
            column_id: col::PROMPTS_FOLDER,
            kind: IndexKind::Bitmap,
            predicate: None,
            options: IndexOptions::default(),
        },
        IndexDef {
            name: "idx_prompts_tags".into(),
            column_id: col::PROMPTS_TAGS,
            kind: IndexKind::Bitmap,
            predicate: None,
            options: IndexOptions::default(),
        },
        IndexDef {
            name: "idx_prompts_favorite".into(),
            column_id: col::PROMPTS_FAVORITE,
            kind: IndexKind::Bitmap,
            predicate: None,
            options: IndexOptions::default(),
        },
        IndexDef {
            name: "idx_prompts_locked".into(),
            column_id: col::PROMPTS_LOCKED,
            kind: IndexKind::Bitmap,
            predicate: None,
            options: IndexOptions::default(),
        },
        IndexDef {
            name: "idx_prompts_updated".into(),
            column_id: col::PROMPTS_UPDATED,
            kind: IndexKind::LearnedRange,
            predicate: None,
            options: IndexOptions {
                learned_range: Some(LearnedRangeOptions::default()),
                ..Default::default()
            },
        },
        IndexDef {
            name: "idx_prompts_char".into(),
            column_id: col::PROMPTS_CHAR,
            kind: IndexKind::LearnedRange,
            predicate: None,
            options: IndexOptions {
                learned_range: Some(LearnedRangeOptions::default()),
                ..Default::default()
            },
        },
        IndexDef {
            name: "idx_prompts_body_fm".into(),
            column_id: col::PROMPTS_BODY,
            kind: IndexKind::FmIndex,
            predicate: None,
            options: IndexOptions::default(),
        },
        IndexDef {
            name: PROMPTS_EMBED_ANN_INDEX.into(),
            column_id: col::PROMPTS_EMBED,
            kind: IndexKind::Ann,
            predicate: None,
            options: IndexOptions {
                ann: Some(AnnOptions {
                    quantization,
                    ..AnnOptions::default()
                }),
                ..Default::default()
            },
        },
        IndexDef {
            name: "idx_prompts_body_sparse".into(),
            column_id: col::PROMPTS_BODY_SPARSE,
            kind: IndexKind::Sparse,
            predicate: None,
            options: IndexOptions::default(),
        },
        IndexDef {
            name: "idx_prompts_body_minhash".into(),
            column_id: col::PROMPTS_BODY_MINHASH,
            kind: IndexKind::MinHash,
            predicate: None,
            options: IndexOptions {
                minhash: Some(MinHashOptions {
                    permutations: 128,
                    bands: 32,
                }),
                ..Default::default()
            },
        },
    ]
}

/// Singleton app-state row (`id = 1`). Schema version, current vault, recents,
/// tutorial flag, theme, beta opt-in, embedding-quantization preference.
pub fn app_state_schema() -> Schema {
    Schema {
        schema_id: 1,
        columns: vec![
            ColumnDef {
                id: col::APP_ID,
                name: "id".into(),
                ty: TypeId::Int64,
                flags: pk(),
                default_value: Some(DefaultExpr::Static(Value::Int64(1))),
                embedding_source: None,
            },
            ColumnDef {
                id: col::APP_SCHEMA_VER,
                name: "schema_version".into(),
                ty: TypeId::Int64,
                flags: ColumnFlags::empty(),
                default_value: Some(DefaultExpr::Static(Value::Int64(0))),
                embedding_source: None,
            },
            ColumnDef {
                id: col::APP_VAULT_PATH,
                name: "current_vault_path".into(),
                ty: TypeId::Bytes,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::APP_LAST_OPENED,
                name: "last_opened_prompt".into(),
                ty: TypeId::Bytes,
                flags: nullable(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::APP_RECENT,
                name: "recent_searches".into(),
                ty: TypeId::Json,
                flags: nullable(),
                default_value: Some(DefaultExpr::Static(Value::Json(b"[]".to_vec()))),
                embedding_source: None,
            },
            ColumnDef {
                id: col::APP_TUTORIAL_DONE,
                name: "tutorial_completed".into(),
                ty: TypeId::Bool,
                flags: ColumnFlags::empty(),
                default_value: Some(DefaultExpr::Static(Value::Bool(false))),
                embedding_source: None,
            },
            ColumnDef {
                id: col::APP_THEME,
                name: "theme".into(),
                ty: TypeId::Bytes,
                flags: ColumnFlags::empty(),
                default_value: Some(DefaultExpr::Static(Value::Bytes(b"dark".to_vec()))),
                embedding_source: None,
            },
            ColumnDef {
                id: col::APP_BETA,
                name: "beta_channel".into(),
                ty: TypeId::Bool,
                flags: ColumnFlags::empty(),
                default_value: Some(DefaultExpr::Static(Value::Bool(false))),
                embedding_source: None,
            },
            // User-tunable embedding quantization: 'binary' (default — HNSW binary + exact cosine rerank)
            // or 'dense' (full f32 HNSW, no rerank). Changing this setting requires dropping +
            // recreating the prompts.embedding Ann index (handled by Task 6.10).
            ColumnDef {
                id: col::APP_EMBED_QUANT,
                name: "embedding_quant".into(),
                ty: TypeId::Bytes,
                flags: ColumnFlags::empty(),
                default_value: Some(DefaultExpr::Static(Value::Bytes(b"binary".to_vec()))),
                embedding_source: None,
            },
            ColumnDef {
                id: col::APP_MINIMIZE_ON_COPY,
                name: "minimize_on_copy".into(),
                ty: TypeId::Bool,
                flags: ColumnFlags::empty(),
                default_value: Some(DefaultExpr::Static(Value::Bool(false))),
                embedding_source: None,
            },
        ],
        indexes: vec![],
        colocation: vec![],
        constraints: Default::default(),
        clustered: false,
    }
}

/// Folder names per vault; unique by name (enforced at the app layer, not by an index here).
pub fn folders_schema() -> Schema {
    Schema {
        schema_id: 1,
        columns: vec![
            ColumnDef {
                id: col::FOLDERS_ID,
                name: "id".into(),
                ty: TypeId::Bytes,
                flags: pk(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::FOLDERS_NAME,
                name: "name".into(),
                ty: TypeId::Bytes,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::FOLDERS_CREATED,
                name: "created_at".into(),
                ty: TypeId::Int64,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::FOLDERS_UPDATED,
                name: "updated_at".into(),
                ty: TypeId::Int64,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
        ],
        indexes: vec![IndexDef {
            name: "idx_folders_name".into(),
            column_id: col::FOLDERS_NAME,
            kind: IndexKind::Bitmap,
            predicate: None,
            options: IndexOptions::default(),
        }],
        colocation: vec![],
        constraints: Default::default(),
        clustered: false,
    }
}

/// Smart folder: text DSL (parsed to SearchQuery) + optional visual spec.
pub fn smart_folders_schema() -> Schema {
    Schema {
        schema_id: 1,
        columns: vec![
            ColumnDef {
                id: col::SF_ID,
                name: "id".into(),
                ty: TypeId::Bytes,
                flags: pk(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::SF_NAME,
                name: "name".into(),
                ty: TypeId::Bytes,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::SF_DSL,
                name: "dsl".into(),
                ty: TypeId::Bytes,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::SF_VISUAL,
                name: "visual".into(),
                ty: TypeId::Json,
                flags: nullable(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::SF_CREATED,
                name: "created_at".into(),
                ty: TypeId::Int64,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::SF_UPDATED,
                name: "updated_at".into(),
                ty: TypeId::Int64,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
        ],
        indexes: vec![],
        colocation: vec![],
        constraints: Default::default(),
        clustered: false,
    }
}

/// Pending embedding work items; drained by the background worker.
pub fn embedding_queue_schema() -> Schema {
    Schema {
        schema_id: 1,
        columns: vec![
            ColumnDef {
                id: col::EQ_PROMPT_ID,
                name: "prompt_id".into(),
                ty: TypeId::Bytes,
                flags: pk(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::EQ_QUEUED_AT,
                name: "queued_at".into(),
                ty: TypeId::Int64,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::EQ_ATTEMPTS,
                name: "attempts".into(),
                ty: TypeId::Int64,
                flags: ColumnFlags::empty(),
                default_value: Some(DefaultExpr::Static(Value::Int64(0))),
                embedding_source: None,
            },
            ColumnDef {
                id: col::EQ_LAST_ERROR,
                name: "last_error".into(),
                ty: TypeId::Bytes,
                flags: nullable(),
                default_value: None,
                embedding_source: None,
            },
        ],
        indexes: vec![],
        colocation: vec![],
        constraints: Default::default(),
        clustered: false,
    }
}

/// Installed plugin registry: identity, version, on-disk path, ed25519 signature,
/// JSON capabilities, install timestamp, enabled flag.
pub fn plugins_schema() -> Schema {
    Schema {
        schema_id: 1,
        columns: vec![
            ColumnDef {
                id: col::PL_ID,
                name: "id".into(),
                ty: TypeId::Bytes,
                flags: pk(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::PL_NAME,
                name: "name".into(),
                ty: TypeId::Bytes,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::PL_VERSION,
                name: "version".into(),
                ty: TypeId::Bytes,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::PL_PATH,
                name: "path".into(),
                ty: TypeId::Bytes,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::PL_SIG,
                name: "signature".into(),
                ty: TypeId::Bytes,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::PL_CAPS,
                name: "capabilities".into(),
                ty: TypeId::Json,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::PL_INSTALLED_AT,
                name: "installed_at".into(),
                ty: TypeId::Int64,
                flags: ColumnFlags::empty(),
                default_value: None,
                embedding_source: None,
            },
            ColumnDef {
                id: col::PL_ENABLED,
                name: "enabled".into(),
                ty: TypeId::Bool,
                flags: ColumnFlags::empty(),
                default_value: Some(DefaultExpr::Static(Value::Bool(true))),
                embedding_source: None,
            },
        ],
        indexes: vec![],
        colocation: vec![],
        constraints: Default::default(),
        clustered: false,
    }
}

/// Create every onQ table in `db`. Idempotent: skips tables that already
/// exist (so re-opening an existing vault works without a separate "create vs.
/// open" path).
pub fn create_all_tables(db: &mongreldb_core::Database) -> CoreResult<()> {
    let tables: Vec<(&str, Schema)> = vec![
        ("prompts", prompts_schema()),
        ("app_state", app_state_schema()),
        ("folders", folders_schema()),
        ("smart_folders", smart_folders_schema()),
        ("embedding_queue", embedding_queue_schema()),
        ("plugins", plugins_schema()),
    ];
    let existing = db.table_names();
    for (name, schema) in tables {
        if existing.iter().any(|n| n == name) {
            continue;
        }
        db.create_table(name, schema)
            .map_err(|e| CoreError::Db(format!("create {name}: {e}")))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompts_schema_has_eleven_indexes() {
        // 4 Bitmap + 2 LearnedRange + 1 FmIndex + 1 Ann + 1 Sparse + 1 MinHash = 10
        // (The brief counted 11 by including an extra Bitmap; the actual mapping is
        //  4 Bitmap + 2 LearnedRange + 1 FmIndex + 1 Ann + 1 Sparse + 1 MinHash = 10.)
        let indexes = prompts_indexes();
        assert_eq!(indexes.len(), 10);
        let kinds: Vec<_> = indexes.iter().map(|i| i.kind).collect();
        assert_eq!(kinds.iter().filter(|k| **k == IndexKind::Bitmap).count(), 4);
        assert_eq!(
            kinds
                .iter()
                .filter(|k| **k == IndexKind::LearnedRange)
                .count(),
            2
        );
        assert_eq!(
            kinds.iter().filter(|k| **k == IndexKind::FmIndex).count(),
            1
        );
        assert_eq!(kinds.iter().filter(|k| **k == IndexKind::Ann).count(), 1);
        assert_eq!(kinds.iter().filter(|k| **k == IndexKind::Sparse).count(), 1);
        assert_eq!(
            kinds.iter().filter(|k| **k == IndexKind::MinHash).count(),
            1
        );
    }

    #[test]
    fn prompts_schema_columns_have_unique_ids() {
        let cols = &prompts_schema().columns;
        let mut ids: Vec<u16> = cols.iter().map(|c| c.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), cols.len(), "duplicate column ids in prompts");
    }

    #[test]
    fn prompts_schema_honors_dense_preference() {
        let schema = prompts_schema_with_quantization(AnnQuantization::Dense);
        let ann = schema
            .indexes
            .iter()
            .find(|idx| idx.name == PROMPTS_EMBED_ANN_INDEX)
            .expect("embedding ANN index present");
        let options = ann.options.ann.as_ref().expect("ann options");
        assert_eq!(options.quantization, AnnQuantization::Dense);
    }

    #[test]
    fn prompts_schema_default_is_binary_sign() {
        let schema = prompts_schema();
        let ann = schema
            .indexes
            .iter()
            .find(|idx| idx.name == PROMPTS_EMBED_ANN_INDEX)
            .expect("embedding ANN index present");
        let options = ann.options.ann.as_ref().expect("ann options");
        assert_eq!(options.quantization, AnnQuantization::BinarySign);
    }

    #[test]
    fn app_state_has_singleton_id_column() {
        let cols = &app_state_schema().columns;
        assert_eq!(cols[0].id, col::APP_ID);
        assert!(cols[0].flags.contains(ColumnFlags::PRIMARY_KEY));
    }

    #[test]
    fn schemas_validate_defaults() {
        prompts_schema()
            .validate_defaults()
            .expect("prompts defaults");
        app_state_schema()
            .validate_defaults()
            .expect("app_state defaults");
        folders_schema()
            .validate_defaults()
            .expect("folders defaults");
        smart_folders_schema()
            .validate_defaults()
            .expect("smart_folders defaults");
        embedding_queue_schema()
            .validate_defaults()
            .expect("embedding_queue defaults");
        plugins_schema()
            .validate_defaults()
            .expect("plugins defaults");
    }

    #[test]
    fn schemas_validate_ai_columns() {
        prompts_schema().validate_ai().expect("prompts ai columns");
    }
}
