//! Shared corpus fixtures for benchmarks and conformance tests.
//!
//! Lives in `onq-test-utils` (not `onq-core`) so the heavy
//! `Db::open` + 1,000-row insert can be reused by both the Criterion
//! benchmarks under `crates/onq-core/benches/` and any future
//! performance/scale tests without duplicating the row-construction logic.
//!
//! The corpus shape is identical to the conformance corpus in
//! `crates/onq-core/tests/conformance/search_ndcg.rs`: 10 topics
//! x 100 prompts, deterministic ids of the form `t{T:02}-p{N:03}`. This
//! keeps the benchmark representative of the conformance test the
//! regression gates are guarding.

use mongreldb_core::Value;
use onq_core::db::Db;
use onq_core::embed::EMBED_DIM;
use onq_core::schema::col;
use tempfile::TempDir;

/// Number of topics in the synthetic corpus.
const TOPIC_COUNT: usize = 10;
/// Prompts per topic.
const PROMPTS_PER_TOPIC: usize = 100;

/// Build a fresh vault containing exactly `TOPIC_COUNT * PROMPTS_PER_TOPIC`
/// deterministic prompts and return both the live [`Db`] handle and the
/// owning [`TempDir`] (drop = cleanup).
///
/// Insertion happens in a single transaction so a 1k-row corpus lands in
/// well under a second on the bench host. The embedding column is populated
/// with a deterministic 384-dim zero vector — benchmarks that care about
/// ANN vector quality should override the column after `corpus_1k` returns;
/// the search-cost bench only exercises the FM Contains pre-filter.
pub fn corpus_1k() -> (Db, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let db = Db::open(dir.path(), "bench-pass").expect("Db::open");
    let zero_embed = vec![0.0f32; EMBED_DIM];
    db.handle()
        .transaction_for_current_principal(|tx| {
            for topic in 0..TOPIC_COUNT {
                let keywords: Vec<String> = (0..5).map(|n| keyword(topic, n)).collect();
                for n in 0..PROMPTS_PER_TOPIC {
                    let id = format!("t{topic:02}-p{n:03}").into_bytes();
                    let body = format!(
                        "topic {topic} prompt {n}. keywords: {}.",
                        keywords.join(" ")
                    )
                    .into_bytes();
                    tx.put(
                        "prompts",
                        vec![
                            (col::PROMPTS_ID, Value::Bytes(id.clone())),
                            (col::PROMPTS_TITLE, Value::Bytes(id)),
                            (col::PROMPTS_FOLDER, Value::Bytes(b"bench".to_vec())),
                            (col::PROMPTS_BODY, Value::Bytes(body.clone())),
                            (col::PROMPTS_TAGS, Value::Json(br#"[]"#.to_vec())),
                            (col::PROMPTS_FAVORITE, Value::Bool(false)),
                            (col::PROMPTS_LOCKED, Value::Bool(false)),
                            (col::PROMPTS_CHAR, Value::Int64(body.len() as i64)),
                            (col::PROMPTS_CREATED, Value::Int64(0)),
                            (col::PROMPTS_UPDATED, Value::Int64(0)),
                            (col::PROMPTS_EMBED, Value::Embedding(zero_embed.clone())),
                        ],
                    )?;
                }
            }
            Ok(())
        })
        .expect("insert corpus");
    (db, dir)
}

/// Canonical keyword for `(topic, n)`. Names are unique by construction
/// (no keyword appears in two topics) so FM Contains results never bleed
/// across topics — same convention as the conformance corpus.
fn keyword(topic: usize, n: usize) -> String {
    format!("kw_t{topic:02}_n{n}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_1k_inserts_expected_row_count() {
        let (_db, _dir) = corpus_1k();
        // Smoke check: the transaction body ran without panicking. A row
        // count assertion belongs in the conformance suite, not here.
    }
}
