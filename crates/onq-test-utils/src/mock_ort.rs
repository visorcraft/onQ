use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use onq_core::embed::EMBED_DIM;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Deterministic 384-dim stand-in for [`onq_core::embed::Embedder`].
///
/// Tests seed it from a string label so embeddings are reproducible across
/// runs and don't require the real MiniLM model download.
pub struct MockEmbedder {
    seed: u64,
}

impl MockEmbedder {
    /// Hash `seed` (a string label) into the RNG seed used for `embed()`.
    pub fn new(seed: &str) -> Self {
        let mut h = DefaultHasher::new();
        seed.hash(&mut h);
        Self { seed: h.finish() }
    }

    /// Return a fresh L2-normalized `EMBED_DIM`-length vector.
    pub fn embed(&self) -> Vec<f32> {
        let mut rng = StdRng::seed_from_u64(self.seed);
        let mut v: Vec<f32> = (0..EMBED_DIM).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let n: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if n > 0.0 {
            for x in &mut v {
                *x /= n;
            }
        }
        v
    }
}
