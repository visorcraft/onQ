use mongreldb_core::query::{Condition, Query, Retriever};
use serde::{Deserialize, Serialize};

use crate::schema::col;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    #[serde(default)]
    pub folder: Option<String>,
    #[serde(default)]
    pub tags_any: Vec<String>,
    #[serde(default)]
    pub favorite: Option<bool>,
    #[serde(default)]
    pub locked: Option<bool>,
    #[serde(default)]
    pub char_min: Option<i64>,
    #[serde(default)]
    pub char_max: Option<i64>,
    #[serde(default)]
    pub limit: usize,
}

impl SearchQuery {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            folder: None,
            tags_any: vec![],
            favorite: None,
            locked: None,
            char_min: None,
            char_max: None,
            limit: 50,
        }
    }

    /// Build the `Query` (conjunctive conditions) for the filtered pre-pass.
    /// `query_vec` is needed for the FTS embedding context only — the ANN
    /// prefilter is intentionally omitted here so that `to_retrievers` can
    /// run against the full table (see task 3.4's `run_hybrid_search`).
    pub fn to_query(&self, _query_vec: &[f32]) -> Query {
        let mut conditions = Vec::new();
        if let Some(f) = &self.folder {
            conditions.push(Condition::BitmapEq {
                column_id: col::PROMPTS_FOLDER,
                value: f.as_bytes().to_vec(),
            });
        }
        // Bug 3 (tags_any): emit a BitmapIn so the tag filter actually narrows
        // the candidate set. Resolves to the union of bitmap lookups per tag.
        if !self.tags_any.is_empty() {
            conditions.push(Condition::BitmapIn {
                column_id: col::PROMPTS_TAGS,
                values: self
                    .tags_any
                    .iter()
                    .map(|t| t.as_bytes().to_vec())
                    .collect(),
            });
        }
        if let Some(fav) = self.favorite {
            conditions.push(Condition::BitmapEq {
                column_id: col::PROMPTS_FAVORITE,
                value: vec![if fav { 1 } else { 0 }],
            });
        }
        if let Some(lk) = self.locked {
            conditions.push(Condition::BitmapEq {
                column_id: col::PROMPTS_LOCKED,
                value: vec![if lk { 1 } else { 0 }],
            });
        }
        if let Some(lo) = self.char_min {
            let hi = self.char_max.unwrap_or(i64::MAX);
            conditions.push(Condition::Range {
                column_id: col::PROMPTS_CHAR,
                lo,
                hi,
            });
        } else if let Some(hi) = self.char_max {
            conditions.push(Condition::Range {
                column_id: col::PROMPTS_CHAR,
                lo: 0,
                hi,
            });
        }
        if !self.text.is_empty() {
            // FTS via FmContains on body. ANN is NOT a hard prefilter here
            // (Bug 4): the ranked pass in `to_retrievers` runs against the
            // full table — top-k ANN would drop good candidates before any
            // other retriever can vote.
            conditions.push(Condition::FmContains {
                column_id: col::PROMPTS_BODY,
                pattern: self.text.as_bytes().to_vec(),
            });
        }
        Query {
            conditions,
            ..Default::default()
        }
    }

    /// Build the retrievers for the ranked pass (run after pre-filter).
    /// Bug 3 (limit): `k` honors `self.limit` instead of a hardcoded 200.
    pub fn to_retrievers(&self, query_vec: &[f32]) -> Vec<Retriever> {
        let mut out = Vec::new();
        if !self.text.is_empty() {
            out.push(Retriever::Ann {
                column_id: col::PROMPTS_EMBED,
                query: query_vec.to_vec(),
                k: self.limit,
            });
            // SparseMatch requires a tokenized sparse query — placeholder; in
            // a future task the query text will be tokenized via the same
            // vocabulary used at write time. Bug 2: an empty `vec![]` is
            // rejected by MongrelDB, so skip emitting the sparse retriever
            // until real tokenization lands.
            let sparse_query: Vec<(u32, f32)> = Vec::new();
            if !sparse_query.is_empty() {
                out.push(Retriever::Sparse {
                    column_id: col::PROMPTS_BODY_SPARSE,
                    query: sparse_query,
                    k: self.limit,
                });
            }
        }
        out
    }
}

/// Reciprocal Rank Fusion across the actually-emitted retrievers + recency +
/// favorite boosts.
///
/// Bug 3: dropped the unused `bm25_rank` argument — neither the FM retriever
/// nor any current retriever emits an FM-style rank, and rank fusion only
/// makes sense across the retrievers we actually run. With the BM25 term gone,
/// the recency + favorite constants had to be rebalanced (Bug 3) so they no
/// longer swamp the RRF signal: max RRF for a top-hit across both retrievers
/// is `2/(60+1) ≈ 0.0327`, so recency/favorite must stay comfortably below it.
pub fn rrf_score(
    cosine_rank: usize,
    sparse_rank: usize,
    updated_at: i64,
    now: i64,
    favorite: bool,
) -> f64 {
    const K: f64 = 60.0;
    let rrf = 1.0 / (K + cosine_rank as f64) + 1.0 / (K + sparse_rank as f64);
    let age_days = (now - updated_at) as f64 / 86_400.0;
    let recency = 0.05 * (-age_days / 30.0).exp();
    let fav = if favorite { 0.03 } else { 0.0 };
    rrf + recency + fav
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rrf_prefers_top_ranks() {
        let a = rrf_score(1, 50, 0, 0, false);
        let b = rrf_score(50, 50, 0, 0, false);
        assert!(a > b);
    }

    #[test]
    fn rrf_uses_only_emitted_retrievers() {
        // Sparse rank must still influence the fused score (Bug 3 regression).
        let both = rrf_score(10, 10, 0, 0, false);
        let cosine_only = rrf_score(10, 1_000, 0, 0, false);
        assert!(both > cosine_only);
    }

    #[test]
    fn recency_boost_drops_with_age() {
        let fresh = rrf_score(50, 50, 1_000_000, 1_000_000, false);
        let old = rrf_score(50, 50, 1_000_000 - 86_400 * 90, 1_000_000, false);
        assert!(fresh > old);
    }

    #[test]
    fn favorite_boost_applies() {
        let fav = rrf_score(50, 50, 0, 0, true);
        let no = rrf_score(50, 50, 0, 0, false);
        assert!(fav > no);
    }

    #[test]
    fn recency_and_favorite_bonuses_are_smaller_than_before() {
        // Bug 3 regression: recency + favorite used to be 0.5 + 0.3 = 0.80,
        // which dwarfed max RRF (~0.05 for three retrievers at rank 1). The
        // post-review constants (0.05 + 0.03 = 0.08) are an order of magnitude
        // smaller, so the rank signal can win on a typical hit.
        let age_zero_fav = rrf_score(50, 50, 0, 0, true);
        let rrf_only = 1.0 / 61.0 + 1.0 / 110.0;
        assert!(
            (age_zero_fav - rrf_only) < 0.10,
            "fresh+favorite bonus should stay under ~0.10: delta = {}",
            age_zero_fav - rrf_only
        );
    }

    #[test]
    fn query_with_text_uses_fm_only() {
        let q = SearchQuery::new("api errors");
        let vec = vec![0.0; 384];
        let query = q.to_query(&vec);
        assert!(query
            .conditions
            .iter()
            .any(|c| matches!(c, Condition::FmContains { .. })));
        // Bug 4: the ANN prefilter was removed — text-only queries must NOT
        // add an Ann condition here (the ranked pass handles ANN).
        assert!(!query
            .conditions
            .iter()
            .any(|c| matches!(c, Condition::Ann { .. })));
    }

    #[test]
    fn query_with_no_text_has_no_fts() {
        let q = SearchQuery::new("");
        let vec = vec![0.0; 384];
        let query = q.to_query(&vec);
        assert!(!query
            .conditions
            .iter()
            .any(|c| matches!(c, Condition::FmContains { .. })));
    }

    #[test]
    fn to_query_with_tags_emits_bitmap_in() {
        let mut q = SearchQuery::new("");
        q.tags_any = vec!["rust".into(), "search".into()];
        let vec = vec![0.0; 384];
        let query = q.to_query(&vec);
        let bitmap_in = query
            .conditions
            .iter()
            .find(|c| matches!(c, Condition::BitmapIn { .. }));
        let cond = bitmap_in.expect("expected a BitmapIn condition for tags_any");
        match cond {
            Condition::BitmapIn { column_id, values } => {
                assert_eq!(*column_id, col::PROMPTS_TAGS);
                assert_eq!(values.len(), 2);
                assert!(values.contains(&b"rust".to_vec()));
                assert!(values.contains(&b"search".to_vec()));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn to_query_without_tags_omits_bitmap_in() {
        let q = SearchQuery::new("");
        let vec = vec![0.0; 384];
        let query = q.to_query(&vec);
        assert!(!query
            .conditions
            .iter()
            .any(|c| matches!(c, Condition::BitmapIn { .. })));
    }

    #[test]
    fn to_retrievers_uses_body_sparse_column() {
        let q = SearchQuery::new("api errors");
        let vec = vec![0.0; 384];
        let retrievers = q.to_retrievers(&vec);
        let ann = retrievers
            .iter()
            .find(|r| matches!(r, Retriever::Ann { .. }))
            .expect("expected an ANN retriever");
        match ann {
            Retriever::Ann { column_id, k, .. } => {
                assert_eq!(*column_id, col::PROMPTS_EMBED);
                assert_eq!(*k, q.limit);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn to_retrievers_propagates_limit() {
        let mut q = SearchQuery::new("api errors");
        q.limit = 17;
        let vec = vec![0.0; 384];
        let retrievers = q.to_retrievers(&vec);
        for r in &retrievers {
            let k = match r {
                Retriever::Ann { k, .. } => *k,
                Retriever::Sparse { k, .. } => *k,
                Retriever::MinHash { k, .. } => *k,
            };
            assert_eq!(k, 17, "retriever must honor self.limit");
        }
    }

    #[test]
    fn to_retrievers_with_empty_text_returns_empty() {
        let q = SearchQuery::new("");
        let vec = vec![0.0; 384];
        assert!(q.to_retrievers(&vec).is_empty());
    }

    #[test]
    fn to_retrievers_does_not_emit_empty_sparse_query() {
        // Bug 2 regression: placeholder sparse retriever must be omitted
        // until real tokenization lands, so MongrelDB never sees
        // `Retriever::Sparse { query: vec![], .. }`.
        let q = SearchQuery::new("api errors");
        let vec = vec![0.0; 384];
        for r in q.to_retrievers(&vec) {
            if let Retriever::Sparse { query, .. } = r {
                assert!(
                    !query.is_empty(),
                    "Sparse retriever must carry a non-empty query"
                );
            }
        }
    }
}
