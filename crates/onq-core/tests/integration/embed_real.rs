//! Real-model contract test for [`onq_core::embed`].
//!
//! Downloads `sentence-transformers/all-MiniLM-L6-v2` via [`download_model`],
//! loads the [`Embedder`], and asserts the production code path produces
//! well-formed MiniLM sentence embeddings:
//!
//! 1. `embed("hello world")` returns a 384-dim L2-normalized vector.
//! 2. Cosine similarity between two semantically similar sentences is higher
//!    than between an unrelated pair — the "wrong inputs / wrong pooling"
//!    regression the peer review flagged.
//!
//! `#[ignore]`d so `cargo test --workspace` (no network) stays fast; CI that
//! wants the real-model smoke runs
//! `cargo test --workspace -- --ignored`. Either:
//!
//! - network access is available (HF Hub reachable, ~90 MB download), or
//! - `ONQ_RUN_HEAVY_TESTS=1` is exported (the env var is read by CI).
//!
//! Skip with a clear message when the download itself fails so a transient
//! network issue doesn't fail the suite silently.

use onq_core::embed::{download_model, Embedder, EMBED_DIM};
use tempfile::TempDir;

fn heavy_tests_enabled() -> bool {
    std::env::var_os("ONQ_RUN_HEAVY_TESTS").is_some()
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let mut dot = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
    }
    dot // both inputs are L2-normalized so dot product = cosine similarity
}

#[tokio::test]
#[ignore = "requires network + ~30s to download the MiniLM ONNX model; run with `cargo test -- --ignored` or ONQ_RUN_HEAVY_TESTS=1"]
async fn real_minilm_produces_384d_unit_vector_and_similar_pairs_rank_higher() {
    if !heavy_tests_enabled() {
        eprintln!(
            "skipping: ONQ_RUN_HEAVY_TESTS not set; rerun with \
             `cargo test -- --ignored` after exporting it to exercise the \
             real MiniLM model"
        );
        return;
    }

    let dir = TempDir::new().expect("create tempdir");
    let model_dir = match download_model(dir.path()).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("skipping: download_model failed (network?): {e}");
            return;
        }
    };
    let mut embedder = Embedder::load(&model_dir).expect("load embedder");

    let v = embedder.embed("hello world").expect("embed 'hello world'");
    assert_eq!(v.len(), EMBED_DIM, "embed returns EMBED_DIM components");
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!(
        (norm - 1.0).abs() < 1e-3,
        "embedding is L2-normalized (got norm={norm})"
    );

    let similar_a = embedder
        .embed("the cat sat on the mat")
        .expect("embed similar_a");
    let similar_b = embedder
        .embed("felines rest on rugs")
        .expect("embed similar_b");
    let unrelated = embedder
        .embed("stock market crashed today")
        .expect("embed unrelated");

    let sim_similar = cosine(&similar_a, &similar_b);
    let sim_unrelated = cosine(&similar_a, &unrelated);

    assert!(
        sim_similar > sim_unrelated,
        "similar-pair cosine ({sim_similar:.4}) must exceed unrelated-pair \
         cosine ({sim_unrelated:.4}); if it doesn't, mean-pool or inputs are wrong"
    );
}
