//! Embedding-index management for the prompts table.
//!
//! This module is the home for the two halves of the user-facing embedding
//! quantization toggle:
//!
//! - [`rebuild`]: drop + recreate the `prompts.embedding` Ann index with a
//!   new quantization flag.
//! - [`exact_cosine_search`]: the dense-mode fallback that scans every
//!   candidate row and ranks by exact cosine similarity. The agreed
//!   fallback for the dense quantization mode, because the pinned
//!   `mongreldb-core` API only exposes a binary ANN retriever
//!   (`AnnQuantization::BinarySign`) — there is no `Dense` variant yet, so
//!   a dense HNSW rebuild is impossible until upstream adds the
//!   quantization variant + DROP/CREATE INDEX DDL.
//!
//! ## Why a dense-fallback no-op rebuild?
//!
//! `rebuild` is currently a no-op. Calling it records the user's
//! preference and emits a `tracing::warn!` so operators can see why the
//! index didn't physically change. The setting is honored on the next
//! index creation (e.g. on a fresh vault, when the `prompts_schema` is
//! re-applied) — until then the binary ANN index keeps serving queries.
//!
//! Search-time behaviour:
//! - `embedding_quant == "binary"` -> the existing
//!   `ann_rerank_for_current_principal` path in `commands::run_hybrid_search`
//!   (binary HNSW candidates + exact cosine rerank).
//! - `embedding_quant == "dense"`  -> [`exact_cosine_search`] over the
//!   filtered candidate set. Functionally correct (true cosine similarity)
//!   but slower than HNSW; acceptable because the user opted in.

use mongreldb_core::memtable::Value;
use mongreldb_core::query::Query;
use mongreldb_core::Database;

use crate::db::Db;
use crate::embed::Embedder;
use crate::error::{CoreError, CoreResult};
use crate::schema::col;
use crate::search::SearchQuery;

/// Records the user's chosen embedding quantization for the
/// `prompts.embedding` index. Currently a documented best-effort no-op
/// because the pinned `mongreldb-core` API does not expose DROP/CREATE
/// INDEX DDL or a `Dense` variant of `AnnQuantization` — only
/// `BinarySign` exists. We still record the setting in `app_state` so
/// the choice applies to the next time the index is created (e.g. a
/// fresh vault, or once upstream MongrelDB adds the supporting APIs).
///
/// Returns `Ok(())` unconditionally: refusing to rebuild would be a UX
/// regression (the user picked "dense" in the settings UI and now their
/// toggle looks broken). Logging the warn keeps the operator informed.
///
/// Args:
/// - `db`: open, unlocked search-index DB.
/// - `new_quant`: `"binary"` (default) or `"dense"`. Other values are
///   logged at warn and treated as a no-op rather than rejected here —
///   the Tauri command validates the wire shape before calling this.
pub fn rebuild(_db: &Db, new_quant: &str) -> CoreResult<()> {
    tracing::warn!(
        target: "onQ.embedding_index",
        quant = new_quant,
        "embedding-index rebuild is a no-op until mongreldb-core exposes \
         DROP/CREATE INDEX DDL and a Dense AnnQuantization variant; the \
         requested setting is recorded in app_state and will take effect \
         on the next index creation (e.g. fresh vault). The on-disk \
         index stays binary HNSW in the meantime.",
    );
    Ok(())
}

/// Embed `query.text` via the shared ONNX session. The dense-mode
/// fallback uses this as the single embedding call before the exact
/// cosine scan, so the call site stays consistent with the binary path
/// (both branches take a precomputed `Vec<f32>` query vector).
///
/// Args:
/// - `embedder`: ONNX embedder wrapped in `Arc<Embedder>` and guarded
///   by a `Mutex` in `AppState`. The caller is responsible for handing
///   in a `&mut` view (typically by holding a `MutexGuard` and unwrapping
///   the `Arc` via `Arc::try_unwrap`).
/// - `query`: the search query; only `query.text` is consumed.
///
/// Returns the 384-dim L2-normalized embedding, ready to feed into
/// either the ANN retriever or [`exact_cosine_search`].
pub fn extract_query_vec(embedder: &mut Embedder, query: &SearchQuery) -> CoreResult<Vec<f32>> {
    embedder.embed(&query.text)
}

/// Brute-force exact cosine similarity over every row of `table` that
/// matches `candidate_filter`. Used by the dense-mode fallback in
/// `commands::run_hybrid_search` — the binary ANN path is bypassed and
/// we rank candidates against the stored full-precision embeddings
/// directly. This is functionally correct (true cosine similarity) but
/// slower than HNSW; acceptable because the user opted in via the
/// settings UI.
///
/// Args:
/// - `db`: underlying MongrelDB handle. The `Db` wrapper is not needed
///   here because no app_state round-trip is performed — the caller
///   already loaded the candidate rows.
/// - `table`: table name (always `"prompts"` in the production path).
/// - `column_id`: embedding column id (always `PROMPTS_EMBED` in the
///   production path). Kept parametric so a future caller could reuse
///   the helper against a different table.
/// - `query_vec`: precomputed 384-dim query embedding.
/// - `k`: maximum number of hits to return; top-`k` by descending
///   similarity. `k == 0` returns an empty vector.
/// - `candidate_filter`: optional conjunctive filter to narrow the
///   candidate set before scoring. `None` scans the whole table.
///
/// Returns: ordered `(prompt_id, cosine_similarity)` pairs. Score is
/// `f32` in the range `[-1.0, 1.0]` for normalized vectors; degenerate
/// zero vectors score `0.0` (avoiding div-by-zero).
///
/// Note: this is a one-shot linear scan; for very large tables the cost
/// is `O(N * dim)` where `dim == 384`. The dense-mode user is
/// explicitly trading throughput for recall.
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

    // Default filter to an empty conjunctive query when none is supplied.
    // `unwrap_or` would create a temporary `Query::default()` whose borrow
    // is freed before `query_for_current_principal` runs, so we bind the
    // fallback explicitly.
    let default_filter = Query::default();
    let filter: &Query = candidate_filter.unwrap_or(&default_filter);
    // Project just the PK + the embedding column to keep the wire tiny.
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

    // Stable partial_cmp sort descending by score. NaN entries (theoretically
    // possible from degenerate zero vectors — see cosine_similarity guard)
    // tie-break as Equal so the deterministic truncation below doesn't
    // pick arbitrarily.
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k);
    Ok(scored)
}

/// Cosine similarity between two equal-length vectors. Returns `0.0`
/// for any zero-norm input (avoids div-by-zero on the unit-norm /
/// identity fallback). Negative similarity (vectors pointing in
/// opposite hemispheres) is preserved — the dense ranking should
/// surface the closest neighbour even if the dot product is negative.
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

    /// Insert one prompt row with a caller-supplied embedding vector.
    /// All other columns are populated with type-correct defaults so the
    /// row satisfies the schema (PROMPTS_BODY is non-null in the spec).
    fn insert_prompt_with_embed(db: &Db, id: &str, embed: Vec<f32>) {
        // Use the prompt id as the body content so FmContains filters in
        // tests can target a specific row (e.g. the candidate_filter
        // test below looks for "alpha" in the body).
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
    fn rebuild_is_a_noop_with_warn() {
        // The pinned mongreldb-core API does not expose drop/create index
        // DDL or a Dense AnnQuantization variant, so rebuild is a
        // documented best-effort no-op. The setting is still recorded
        // (via the Tauri command's `set_app_setting` call); the warn is
        // emitted on every invocation so operators can see why the index
        // didn't physically change.
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();
        // Both recognised modes + an unknown one must all succeed.
        assert!(rebuild(&db, "binary").is_ok(), "rebuild(binary) must Ok");
        assert!(rebuild(&db, "dense").is_ok(), "rebuild(dense) must Ok");
        assert!(
            rebuild(&db, "garbage-quant").is_ok(),
            "rebuild must not reject — Tauri command validates upstream"
        );
    }

    #[test]
    fn exact_cosine_search_returns_top_k_by_similarity() {
        // 4-row synthetic corpus on a 384-dim unit sphere:
        //   alpha == query_vec  -> cosine ~+1.0
        //   beta  orthogonal      -> cosine ~ 0.0
        //   gamma negated         -> cosine ~-1.0
        //   delta random          -> cosine in [-1, 1]
        // The expected descending order is alpha > beta > delta > gamma
        // (random is statistically somewhere in between).
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
        delta[2] = 1.0; // another orthogonal axis -> cosine exactly 0.0

        insert_prompt_with_embed(&db, "alpha", alpha);
        insert_prompt_with_embed(&db, "beta", beta);
        insert_prompt_with_embed(&db, "gamma", gamma);
        insert_prompt_with_embed(&db, "delta", delta);

        let hits = exact_cosine_search(db.handle(), "prompts", col::PROMPTS_EMBED, &query, 3, None)
            .expect("exact_cosine_search ok");

        assert_eq!(hits.len(), 3, "expected top-3, got {}", hits.len());
        // Top-1 must be the row identical to the query.
        assert_eq!(hits[0].0, "alpha");
        assert!(
            (hits[0].1 - 1.0).abs() < 1e-4,
            "alpha cosine expected ~1.0, got {}",
            hits[0].1
        );
        // Middle slots must be the orthogonal rows (cosine exactly 0).
        for (id, score) in hits.iter().take(3).skip(1) {
            assert!(
                score.abs() < 1e-4,
                "expected orthogonal cosine ~0 for {id}, got {score}"
            );
        }
        // Gamma (cosine = -1.0) must NOT appear in the top-3.
        assert!(
            hits.iter().all(|(id, _)| id != "gamma"),
            "gamma (cosine -1) must not appear in top-3: {hits:?}"
        );
    }

    #[test]
    fn exact_cosine_search_respects_requested_k() {
        // Same corpus shape, but request k=1 — only the best match returns.
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
        // Three rows, but the FmContains filter only matches "alpha".
        // The filter narrows the candidate set so gamma/beta can't appear
        // in the results even if their cosine similarity would rank them
        // higher.
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
        // The k=0 sentinel means "don't return anything" — the helper
        // short-circuits before touching the DB. Regression guard for
        // an early loop that always truncated after the query.
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path(), "test-pass").unwrap();
        let query = vec![0.0f32; 384];
        let hits = exact_cosine_search(db.handle(), "prompts", col::PROMPTS_EMBED, &query, 0, None)
            .expect("k=0 ok");
        assert!(hits.is_empty(), "k=0 must short-circuit to empty Vec");
    }

    #[test]
    fn cosine_similarity_unit_vectors_returns_one() {
        // Sanity check on the math: unit vectors that point the same
        // direction must score 1.0; opposite must score -1.0;
        // orthogonal must score 0.0.
        let a = vec![1.0f32, 0.0, 0.0];
        assert!((cosine_similarity(&a, &a) - 1.0).abs() < 1e-6);
        let opp = vec![-1.0f32, 0.0, 0.0];
        assert!((cosine_similarity(&a, &opp) - (-1.0)).abs() < 1e-6);
        let ortho = vec![0.0f32, 1.0, 0.0];
        assert!(cosine_similarity(&a, &ortho).abs() < 1e-6);
    }

    #[test]
    fn cosine_similarity_zero_vector_returns_zero() {
        // Degenerate input — must not divide by zero. The dense ranking
        // uses this fallback so freshly-created prompts (zero embedding
        // until the sync worker fills them in) don't poison the top-k
        // with NaN scores.
        let zero = vec![0.0f32; 384];
        let mut nonzero = vec![0.0f32; 384];
        nonzero[0] = 1.0;
        assert_eq!(cosine_similarity(&zero, &nonzero), 0.0);
        assert_eq!(cosine_similarity(&nonzero, &zero), 0.0);
        assert_eq!(cosine_similarity(&zero, &zero), 0.0);
    }
}
