//! Criterion benchmark for the FM Contains pre-filter over the 1k corpus.
//!
//! Measures the wall-clock cost of a warm search hit on the corpus built by
//! [`onq_test_utils::fixtures::corpus_1k`]. The fixture seeds 1,000
//! prompts across 10 topics; the bench issues a single-keyword query that
//! should match all 100 prompts of `topic 0`, exercising the same code path
//! the search palette hits on every keystroke (FM Contains -> result list).
//!
//! ANN rerank is intentionally out of scope here: it requires a real
//! embedding query vector, and the corpus ships with a zero vector for the
//! embedding column (see `fixtures::corpus_1k`). The benchmark tracks the
//! pre-filter cost, which dominates tail latency for cold-cache / wide-result
//! queries. A future task can add an `ann_rerank` variant once the corpus
//! fixture learns how to seed realistic vectors.

use criterion::{criterion_group, criterion_main, Criterion};
use onq_core::search::SearchQuery;
use onq_test_utils::fixtures::corpus_1k;

/// The keyword that exists in exactly the 100 prompts of `topic 0`. See
/// `onq_test_utils::fixtures::corpus_1k` for the keyword naming.
const TOPIC0_KEYWORD: &str = "kw_t00_n0";

fn bench_search(c: &mut Criterion) {
    let (db, _dir) = corpus_1k();
    c.bench_function("search_warm_p95", |b| {
        b.iter(|| {
            let q = SearchQuery::new(TOPIC0_KEYWORD);
            let query_vec = vec![0.0f32; onq_core::embed::EMBED_DIM];
            let query = q.to_query(&query_vec);
            // The pre-filter is the part the search palette waits on;
            // skipping the ANN rerank keeps the bench CPU-bound and
            // deterministic without dragging in a real embedder.
            let _rows = db
                .handle()
                .query_for_current_principal("prompts", &query, None)
                .expect("search query");
        });
    });
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
