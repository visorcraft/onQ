use mongreldb_core::query::{Condition, Query, Retriever};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

    /// Build only structured hard filters. Free text belongs in ranked
    /// retrievers; making it an `FmContains` condition would discard semantic
    /// matches that do not contain the exact query text.
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
            if !query_vec.is_empty() {
                out.push(Retriever::Ann {
                    column_id: col::PROMPTS_EMBED,
                    query: query_vec.to_vec(),
                    k: self.limit,
                });
            }
            let sparse_query = sparse_vector(&self.text);
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

/// Build the free-text document string used for sparse (and ideally dense)
/// indexing. Title, tags, and folder are included so typing a tag or project
/// name in the palette can surface matching prompts — not only body tokens.
pub fn searchable_text(
    title: &str,
    tags: &[String],
    folder: Option<&str>,
    body: &str,
) -> String {
    let mut parts: Vec<&str> = Vec::with_capacity(4 + tags.len());
    let title = title.trim();
    if !title.is_empty() {
        parts.push(title);
    }
    for tag in tags {
        let t = tag.trim();
        if !t.is_empty() {
            parts.push(t);
        }
    }
    if let Some(folder) = folder.map(str::trim).filter(|f| !f.is_empty()) {
        parts.push(folder);
    }
    let body = body.trim();
    if !body.is_empty() {
        parts.push(body);
    }
    parts.join("\n")
}

/// Deterministic hashing-trick sparse vector shared by document writes and
/// queries. L2-normalized term frequency prevents long prompts from winning
/// only because they contain more words.
pub fn sparse_vector(text: &str) -> Vec<(u32, f32)> {
    let mut terms = BTreeMap::<u32, f32>::new();
    for token in text
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
    {
        let mut hash = 0x811c9dc5u32;
        for byte in token.to_lowercase().bytes() {
            hash = (hash ^ u32::from(byte)).wrapping_mul(0x01000193);
        }
        *terms.entry(hash).or_default() += 1.0;
    }
    let norm = terms
        .values()
        .map(|weight| weight * weight)
        .sum::<f32>()
        .sqrt();
    if norm > 0.0 {
        for weight in terms.values_mut() {
            *weight /= norm;
        }
    }
    terms.into_iter().collect()
}

pub fn sparse_bytes(text: &str) -> Option<Vec<u8>> {
    let vector = sparse_vector(text);
    (!vector.is_empty()).then(|| {
        mongreldb_core::query::encode_sparse_vector(&vector)
            .expect("sparse vectors are serializable")
    })
}

/// Reciprocal Rank Fusion across retrievers that actually returned the row,
/// plus small recency and favorite tie-breakers.
pub fn rrf_score(
    cosine_rank: Option<usize>,
    sparse_rank: Option<usize>,
    updated_at: i64,
    now: i64,
    favorite: bool,
) -> f64 {
    const K: f64 = 60.0;
    let rrf = cosine_rank
        .into_iter()
        .chain(sparse_rank)
        .map(|rank| 1.0 / (K + rank as f64))
        .sum::<f64>();
    let age_days = (now - updated_at) as f64 / 86_400.0;
    let recency = 0.005 * (-age_days / 30.0).exp();
    let fav = if favorite { 0.003 } else { 0.0 };
    rrf + recency + fav
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rrf_prefers_top_ranks() {
        let a = rrf_score(Some(1), Some(50), 0, 0, false);
        let b = rrf_score(Some(50), Some(50), 0, 0, false);
        assert!(a > b);
    }

    #[test]
    fn rrf_uses_only_emitted_retrievers() {
        let both = rrf_score(Some(10), Some(10), 0, 0, false);
        let cosine_only = rrf_score(Some(10), None, 0, 0, false);
        assert!(both > cosine_only);
    }

    #[test]
    fn recency_boost_drops_with_age() {
        let fresh = rrf_score(Some(50), Some(50), 1_000_000, 1_000_000, false);
        let old = rrf_score(
            Some(50),
            Some(50),
            1_000_000 - 86_400 * 90,
            1_000_000,
            false,
        );
        assert!(fresh > old);
    }

    #[test]
    fn favorite_boost_applies() {
        let fav = rrf_score(Some(50), Some(50), 0, 0, true);
        let no = rrf_score(Some(50), Some(50), 0, 0, false);
        assert!(fav > no);
    }

    #[test]
    fn recency_and_favorite_bonuses_are_smaller_than_before() {
        let age_zero_fav = rrf_score(Some(50), Some(50), 0, 0, true);
        let rrf_only = 1.0 / 61.0 + 1.0 / 110.0;
        assert!(
            (age_zero_fav - rrf_only) < 0.01,
            "fresh+favorite bonus should stay under 0.01: delta = {}",
            age_zero_fav - rrf_only
        );
    }

    #[test]
    fn free_text_is_not_a_hard_filter() {
        let q = SearchQuery::new("api errors");
        let vec = vec![0.0; 384];
        let query = q.to_query(&vec);
        assert!(!query
            .conditions
            .iter()
            .any(|condition| matches!(condition, Condition::FmContains { .. })));
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
        let sparse = retrievers
            .iter()
            .find(|retriever| matches!(retriever, Retriever::Sparse { .. }))
            .expect("expected a sparse retriever");
        match sparse {
            Retriever::Sparse {
                column_id, query, ..
            } => {
                assert_eq!(*column_id, col::PROMPTS_BODY_SPARSE);
                assert!(!query.is_empty());
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
    fn missing_embedder_keeps_sparse_retrieval() {
        let retrievers = SearchQuery::new("api errors").to_retrievers(&[]);
        assert_eq!(retrievers.len(), 1);
        assert!(matches!(retrievers[0], Retriever::Sparse { .. }));
    }

    #[test]
    fn sparse_vectors_are_shared_and_normalized() {
        let document = sparse_vector("Rust rust search");
        let query = sparse_vector("rust");
        assert!(document.iter().any(|(token, _)| *token == query[0].0));
        let norm = document
            .iter()
            .map(|(_, weight)| weight * weight)
            .sum::<f32>();
        assert!((norm - 1.0).abs() < f32::EPSILON);
        assert_eq!(sparse_vector(""), Vec::new());
    }

    #[test]
    fn searchable_text_includes_title_tags_folder_and_body() {
        let text = searchable_text(
            "Weekly status",
            &["git".into(), "release".into()],
            Some("Work/Updates"),
            "Ship the build.",
        );
        assert!(text.contains("Weekly status"));
        assert!(text.contains("git"));
        assert!(text.contains("release"));
        assert!(text.contains("Work/Updates"));
        assert!(text.contains("Ship the build."));
        // Tag-only query tokens must appear in the sparse vector of the doc.
        let doc = sparse_vector(&text);
        let q = sparse_vector("git");
        assert!(!q.is_empty());
        let q_hash = q[0].0;
        assert!(
            doc.iter().any(|(h, _)| *h == q_hash),
            "tag token must be present in document sparse vector"
        );
    }
}
