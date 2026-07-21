//! Embedding-index management for the prompts table.
//!
//! User-facing quantization toggle (`embedding_quant` in `app_state`):
//!
//! - [`rebuild`]: durable `replace_index` of `idx_prompts_embed_ann` with the
//!   requested [`AnnQuantization`].
//! - [`dense_readiness`]: whether the live ANN index already matches a Dense
//!   preference (native HNSW) or is still BinarySign / mid-replacement
//!   (exact-cosine fallback).
//! - [`exact_cosine_search`]: brute-force cosine over stored f32 embeddings —
//!   used **only** while a requested Dense replacement has not completed.
//!
//! Once Dense is published, search uses the native ANN path. ANN/auth/resource
//! errors are not papered over with the exact-cosine fallback.

use mongreldb_core::memtable::Value;
use mongreldb_core::query::Query;
use mongreldb_core::schema::{AnnOptions, AnnQuantization, IndexDef, IndexKind, IndexOptions};
use mongreldb_core::Database;

use crate::db::Db;
use crate::embed::Embedder;
use crate::error::{CoreError, CoreResult};
use crate::schema::{col, prompts_indexes_with_quantization, PROMPTS_EMBED_ANN_INDEX};
use crate::search::SearchQuery;

/// Whether a Dense preference can use the live ANN index, or must fall back
/// to exact cosine until replace-index publishes Dense.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenseReadiness {
    /// Live `prompts` ANN index is `AnnQuantization::Dense`.
    Ready,
    /// Preference is dense but the published index is still BinarySign (or
    /// the ANN index is missing) — use exact cosine until rebuild finishes.
    PendingExactFallback,
}

/// Parse the wire / app_state quant string into an ANN quantization.
pub fn parse_quantization(new_quant: &str) -> CoreResult<AnnQuantization> {
    match new_quant {
        "binary" => Ok(AnnQuantization::BinarySign),
        "dense" => Ok(AnnQuantization::Dense),
        other => Err(CoreError::Db(format!(
            "embedding quant must be 'binary' or 'dense', got '{other}'"
        ))),
    }
}

/// Read the published ANN quantization for `idx_prompts_embed_ann` from the
/// authoritative catalog. `None` if the prompts table or ANN index is absent.
pub fn live_ann_quantization(db: &Db) -> CoreResult<Option<AnnQuantization>> {
    live_ann_quantization_on(db.handle())
}

fn live_ann_quantization_on(db: &Database) -> CoreResult<Option<AnnQuantization>> {
    let catalog = db.catalog_snapshot();
    let Some(entry) = catalog.live("prompts") else {
        return Ok(None);
    };
    for index in &entry.schema.indexes {
        if index.name == PROMPTS_EMBED_ANN_INDEX {
            return Ok(index.options.ann.as_ref().map(|ann| ann.quantization));
        }
    }
    Ok(None)
}

/// Dense readiness for hybrid search. Call only when the user preference is
/// `"dense"` — BinarySign preference always uses the binary ANN + rerank path.
pub fn dense_readiness(db: &Db) -> CoreResult<DenseReadiness> {
    match live_ann_quantization(db)? {
        Some(AnnQuantization::Dense) => Ok(DenseReadiness::Ready),
        Some(AnnQuantization::BinarySign) | None => Ok(DenseReadiness::PendingExactFallback),
    }
}

/// Index definition for the prompts embedding ANN at `quantization`.
fn embed_ann_index_def(quantization: AnnQuantization) -> IndexDef {
    prompts_indexes_with_quantization(quantization)
        .into_iter()
        .find(|idx| idx.name == PROMPTS_EMBED_ANN_INDEX)
        .unwrap_or_else(|| IndexDef {
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
        })
}

/// Replace `idx_prompts_embed_ann` with the requested quantization via the
/// durable MongrelDB replace-index job (hidden-generation build + publish).
///
/// No-ops when the live schema already matches. Errors from replace-index
/// (authorization, conflict, resource admission, corruption) propagate —
/// callers must not paper them over with exact-cosine fallback.
///
/// Args:
/// - `db`: open, unlocked search-index DB.
/// - `new_quant`: `"binary"` or `"dense"`.
pub fn rebuild(db: &Db, new_quant: &str) -> CoreResult<()> {
    let target = parse_quantization(new_quant)?;
    let live = live_ann_quantization(db)?;
    if live == Some(target) {
        tracing::debug!(
            target: "onQ.embedding_index",
            ?target,
            "embedding ANN already at requested quantization; skip replace-index"
        );
        return Ok(());
    }

    let new_def = embed_ann_index_def(target);
    tracing::info!(
        target: "onQ.embedding_index",
        ?target,
        ?live,
        "submitting durable replace-index for prompts embedding ANN"
    );
    db.handle()
        .replace_index("prompts", PROMPTS_EMBED_ANN_INDEX, new_def)
        .map_err(|e| CoreError::Db(format!("replace_index {PROMPTS_EMBED_ANN_INDEX}: {e}")))?;
    Ok(())
}

/// On vault open: if `app_state.embedding_quant` disagrees with the live ANN
/// index, submit replace-index so preference and published schema converge.
///
/// Exact-cosine search covers Dense preference while this runs or until the
/// next successful rebuild from Settings.
pub fn reconcile_on_open(db: &Db) -> CoreResult<()> {
    let preferred = db.get_app_setting("embedding_quant")?;
    let preferred = if preferred.is_empty() {
        "binary".to_string()
    } else {
        preferred
    };
    // Unknown wire values are left alone; Tauri validates user toggles.
    if preferred != "binary" && preferred != "dense" {
        tracing::warn!(
            target: "onQ.embedding_index",
            preferred = %preferred,
            "app_state.embedding_quant is not binary|dense; skip reconcile"
        );
        return Ok(());
    }
    rebuild(db, &preferred)
}

/// Embed `query.text` via the shared ONNX session.
pub fn extract_query_vec(embedder: &mut Embedder, query: &SearchQuery) -> CoreResult<Vec<f32>> {
    embedder.embed(&query.text)
}

/// Brute-force exact cosine similarity over every row of `table` that
/// matches `candidate_filter`. Used only while Dense replace-index has not
/// published (`DenseReadiness::PendingExactFallback`).
pub fn exact_cosine_search(
    db: &Database,
    table: &str,
    column_id: u16,
    query_vec: &[f32],
    k: usize,
    candidate_filter: Option<&Query>,
) -> CoreResult<Vec<(String, f32)>> {
    if k == 0 {
        return Ok(Vec::new());
    }

    let default_filter = Query::default();
    let filter: &Query = candidate_filter.unwrap_or(&default_filter);
    let rows = db
        .query_for_current_principal(table, filter, Some(&[col::PROMPTS_ID, column_id]))
        .map_err(|e| CoreError::Db(format!("exact_cosine_search query: {e}")))?;

    let mut scored: Vec<(String, f32)> = rows
        .into_iter()
        .filter_map(|row| {
            let id_bytes = match row.columns.get(&col::PROMPTS_ID) {
                Some(Value::Bytes(b)) => b,
                _ => return None,
            };
            let embed = match row.columns.get(&column_id) {
                Some(Value::Embedding(v)) => v,
                _ => return None,
            };
            let score = cosine_similarity(query_vec, embed);
            let id = String::from_utf8_lossy(id_bytes).into_owned();
            Some((id, score))
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k);
    Ok(scored)
}

/// Cosine similarity between two equal-length vectors. Returns `0.0`
/// for any zero-norm input.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "cosine_similarity: dim mismatch");
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na * nb)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongreldb_core::memtable::Value;
    use mongreldb_core::query::Query;
    use tempfile::TempDir;

    fn insert_prompt_with_embed(db: &Db, id: &str, embed: Vec<f32>) {
        let body = format!("body-of-{id}");
        db.handle()
            .transaction_for_current_principal(|tx| {
                tx.put(
                    "prompts",
                    vec![
                        (col::PROMPTS_ID, Value::Bytes(id.as_bytes().to_vec())),
                        (
                            col::PROMPTS_TITLE,
                            Value::Bytes(format!("title-{id}").into_bytes()),
                        ),
                        (col::PROMPTS_BODY, Value::Bytes(body.as_bytes().to_vec())),
                        (col::PROMPTS_TAGS, Value::Json(br#"[]"#.to_vec())),
                        (col::PROMPTS_FAVORITE, Value::Bool(false)),
                        (col::PROMPTS_LOCKED, Value::Bool(false)),
                        (col::PROMPTS_CHAR, Value::Int64(body.len() as i64)),
                        (col::PROMPTS_CREATED, Value::Int64(0)),
                        (col::PROMPTS_UPDATED, Value::Int64(0)),
                        (col::PROMPTS_EMBED, Value::Embedding(embed)),
                    ],
                )?;
                Ok(())
            })
            .expect("insert prompt");
    }

    #[test]
    fn parse_quantization_accepts_binary_and_dense() {
        assert_eq!(
            parse_quantization("binary").unwrap(),
            AnnQuantization::BinarySign
        );
        assert_eq!(parse_quantization("dense").unwrap(), AnnQuantization::Dense);
        assert!(parse_quantization("garbage").is_err());
    }

    #[test]
    fn fresh_vault_defaults_to_binary_and_pending_when_dense_requested() {
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();
        assert_eq!(
            live_ann_quantization(&db).unwrap(),
            Some(AnnQuantization::BinarySign)
        );
        // Preference not yet dense — readiness for dense path is pending.
        assert_eq!(
            dense_readiness(&db).unwrap(),
            DenseReadiness::PendingExactFallback
        );
    }

    #[test]
    fn rebuild_dense_publishes_dense_ann() {
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();
        // Seed a few embeddings so the index has something to build.
        let mut unit = vec![0.0f32; 384];
        unit[0] = 1.0;
        insert_prompt_with_embed(&db, "alpha", unit.clone());
        insert_prompt_with_embed(&db, "beta", unit);

        rebuild(&db, "dense").expect("rebuild dense");
        assert_eq!(
            live_ann_quantization(&db).unwrap(),
            Some(AnnQuantization::Dense)
        );
        assert_eq!(dense_readiness(&db).unwrap(), DenseReadiness::Ready);

        // Idempotent when already dense.
        rebuild(&db, "dense").expect("rebuild dense again");
        assert_eq!(
            live_ann_quantization(&db).unwrap(),
            Some(AnnQuantization::Dense)
        );

        // Switch back to binary.
        rebuild(&db, "binary").expect("rebuild binary");
        assert_eq!(
            live_ann_quantization(&db).unwrap(),
            Some(AnnQuantization::BinarySign)
        );
        assert_eq!(
            dense_readiness(&db).unwrap(),
            DenseReadiness::PendingExactFallback
        );
    }

    #[test]
    fn reconcile_on_open_honors_saved_preference() {
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();
        db.set_app_setting("embedding_quant", "dense").unwrap();
        reconcile_on_open(&db).expect("reconcile");
        assert_eq!(
            live_ann_quantization(&db).unwrap(),
            Some(AnnQuantization::Dense)
        );
    }

    #[test]
    fn exact_cosine_search_returns_top_k_by_similarity() {
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();

        let mut query = vec![0.0f32; 384];
        query[0] = 1.0;
        let alpha = query.clone();
        let mut beta = vec![0.0f32; 384];
        beta[1] = 1.0;
        let mut gamma = vec![0.0f32; 384];
        gamma[0] = -1.0;
        let mut delta = vec![0.0f32; 384];
        delta[2] = 1.0;

        insert_prompt_with_embed(&db, "alpha", alpha);
        insert_prompt_with_embed(&db, "beta", beta);
        insert_prompt_with_embed(&db, "gamma", gamma);
        insert_prompt_with_embed(&db, "delta", delta);

        let hits = exact_cosine_search(db.handle(), "prompts", col::PROMPTS_EMBED, &query, 3, None)
            .expect("exact_cosine_search ok");

        assert_eq!(hits.len(), 3, "expected top-3, got {}", hits.len());
        assert_eq!(hits[0].0, "alpha");
        assert!(
            (hits[0].1 - 1.0).abs() < 1e-4,
            "alpha cosine expected ~1.0, got {}",
            hits[0].1
        );
        for (id, score) in hits.iter().take(3).skip(1) {
            assert!(
                score.abs() < 1e-4,
                "expected orthogonal cosine ~0 for {id}, got {score}"
            );
        }
        assert!(
            hits.iter().all(|(id, _)| id != "gamma"),
            "gamma (cosine -1) must not appear in top-3: {hits:?}"
        );
    }

    #[test]
    fn exact_cosine_search_respects_requested_k() {
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();

        let mut query = vec![0.0f32; 384];
        query[0] = 1.0;
        let alpha = query.clone();
        insert_prompt_with_embed(&db, "alpha", alpha);
        insert_prompt_with_embed(&db, "beta", vec![0.0; 384]);
        insert_prompt_with_embed(&db, "gamma", vec![0.0; 384]);

        let hits = exact_cosine_search(db.handle(), "prompts", col::PROMPTS_EMBED, &query, 1, None)
            .expect("exact_cosine_search k=1 ok");

        assert_eq!(hits.len(), 1, "k=1 must produce exactly 1 hit");
        assert_eq!(hits[0].0, "alpha", "top-1 must be the identical row");
    }

    #[test]
    fn exact_cosine_search_respects_candidate_filter() {
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();

        let mut query = vec![0.0f32; 384];
        query[0] = 1.0;
        let alpha = query.clone();
        let mut gamma = vec![0.0f32; 384];
        gamma[0] = -1.0;
        let beta = vec![0.0f32; 384];

        insert_prompt_with_embed(&db, "alpha", alpha);
        insert_prompt_with_embed(&db, "beta", beta);
        insert_prompt_with_embed(&db, "gamma", gamma);

        let filter = Query {
            conditions: vec![mongreldb_core::query::Condition::FmContains {
                column_id: col::PROMPTS_BODY,
                pattern: b"alpha".to_vec(),
            }],
            ..Default::default()
        };
        let hits = exact_cosine_search(
            db.handle(),
            "prompts",
            col::PROMPTS_EMBED,
            &query,
            10,
            Some(&filter),
        )
        .expect("filtered exact_cosine_search ok");

        assert_eq!(hits.len(), 1, "filter must narrow to exactly 1 row");
        assert_eq!(hits[0].0, "alpha");
    }

    #[test]
    fn exact_cosine_search_returns_empty_for_k_zero() {
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();
        let query = vec![0.0f32; 384];
        let hits = exact_cosine_search(db.handle(), "prompts", col::PROMPTS_EMBED, &query, 0, None)
            .expect("k=0 ok");
        assert!(hits.is_empty(), "k=0 must short-circuit to empty Vec");
    }

    #[test]
    fn cosine_similarity_unit_vectors_returns_one() {
        let a = vec![1.0f32, 0.0, 0.0];
        assert!((cosine_similarity(&a, &a) - 1.0).abs() < 1e-6);
        let opp = vec![-1.0f32, 0.0, 0.0];
        assert!((cosine_similarity(&a, &opp) - (-1.0)).abs() < 1e-6);
        let ortho = vec![0.0f32, 1.0, 0.0];
        assert!(cosine_similarity(&a, &ortho).abs() < 1e-6);
    }

    #[test]
    fn cosine_similarity_zero_vector_returns_zero() {
        let zero = vec![0.0f32; 384];
        let mut nonzero = vec![0.0f32; 384];
        nonzero[0] = 1.0;
        assert_eq!(cosine_similarity(&zero, &nonzero), 0.0);
        assert_eq!(cosine_similarity(&nonzero, &zero), 0.0);
        assert_eq!(cosine_similarity(&zero, &zero), 0.0);
    }
}
