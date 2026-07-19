//! ONNX-based text embedder.
//!
//! Wraps an `ort` inference [`Session`](ort::session::Session) for the
//! MiniLM transformer plus a HuggingFace [`Tokenizer`]. The `Embedder`
//! itself is the production path; tests use
//! [`onq_test_utils::mock_ort::MockEmbedder`] to avoid the ~90 MB
//! model download + ONNX runtime startup.

use std::path::{Path, PathBuf};

use ort::session::Session;
use ort::value::Tensor;
use tokenizers::Tokenizer;

use crate::error::{CoreError, CoreResult};

pub const MODEL_ID: &str = "sentence-transformers/all-MiniLM-L6-v2";
pub const EMBED_DIM: usize = 384;

/// Sentence-transformer embedder. Owns the loaded ONNX session and tokenizer.
pub struct Embedder {
    session: Session,
    tokenizer: Tokenizer,
}

impl Embedder {
    /// Load an embedder from a local model directory containing
    /// `model.onnx` and `tokenizer.json`.
    pub fn load(model_dir: &Path) -> CoreResult<Self> {
        let session = Session::builder()
            .map_err(|e| CoreError::Other(e.to_string()))?
            .commit_from_file(model_dir.join("model.onnx"))
            .map_err(|e| CoreError::Other(e.to_string()))?;
        let tokenizer = Tokenizer::from_file(model_dir.join("tokenizer.json"))
            .map_err(|e| CoreError::Other(e.to_string()))?;
        Ok(Self { session, tokenizer })
    }

    /// Encode `text` into a single L2-normalized 384-dim `Vec<f32>`.
    pub fn embed(&mut self, text: &str) -> CoreResult<Vec<f32>> {
        let enc = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| CoreError::Other(e.to_string()))?;
        let seq_len = enc.get_ids().len();
        let ids: Vec<i64> = enc.get_ids().iter().map(|i| *i as i64).collect();
        let mask: Vec<i64> = enc.get_attention_mask().iter().map(|i| *i as i64).collect();
        // `sentence-transformers/all-MiniLM-L6-v2` has a single-sequence
        // input, so token-type ids are all zero. The ONNX graph still
        // requires the input to be present.
        let type_ids: Vec<i64> = enc.get_type_ids().iter().map(|i| *i as i64).collect();
        let shape = vec![1i64, seq_len as i64];
        let ids_tensor = Tensor::from_array((shape.clone(), ids))
            .map_err(|e| CoreError::Other(e.to_string()))?;
        let mask_tensor = Tensor::from_array((shape.clone(), mask))
            .map_err(|e| CoreError::Other(e.to_string()))?;
        let type_ids_tensor =
            Tensor::from_array((shape, type_ids)).map_err(|e| CoreError::Other(e.to_string()))?;
        let outputs = self
            .session
            .run(ort::inputs! {
                "input_ids" => ids_tensor,
                "attention_mask" => mask_tensor,
                "token_type_ids" => type_ids_tensor,
            })
            .map_err(|e| CoreError::Other(e.to_string()))?;
        // Output is `last_hidden_state` of shape `[1, seq_len, 384]`.
        // The flat data layout is row-major so token `i`'s 384-dim vector
        // lives at `data[i * 384 .. (i + 1) * 384]`.
        let (out_shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| CoreError::Other(e.to_string()))?;
        if out_shape.len() != 3 || out_shape[0] != 1 || out_shape[2] as usize != EMBED_DIM {
            return Err(CoreError::Other(format!(
                "unexpected model output shape: {out_shape:?}"
            )));
        }
        let attn_mask: Vec<i64> = enc.get_attention_mask().iter().map(|i| *i as i64).collect();
        let mut pooled = mean_pool(data, &attn_mask, EMBED_DIM);
        l2_normalize(&mut pooled);
        Ok(pooled)
    }
}

/// Masked mean-pool over a `[seq_len, dim]` row-major slice using the
/// attention mask (0 for padding, 1 for real tokens). Returns a length-`dim`
/// `Vec<f32>`.
fn mean_pool(data: &[f32], mask: &[i64], dim: usize) -> Vec<f32> {
    debug_assert_eq!(data.len() % dim, 0);
    let seq_len = data.len() / dim;
    debug_assert_eq!(seq_len, mask.len());
    let mut sum = vec![0.0f32; dim];
    let mut real_count = 0.0f32;
    for (i, &m) in mask.iter().take(seq_len).enumerate() {
        if m != 0 {
            real_count += m as f32;
            let row = &data[i * dim..(i + 1) * dim];
            for (s, x) in sum.iter_mut().zip(row.iter()) {
                *s += *x * (m as f32);
            }
        }
    }
    if real_count > 0.0 {
        for s in &mut sum {
            *s /= real_count;
        }
    }
    sum
}

/// In-place L2 normalization; no-op for a zero vector.
fn l2_normalize(v: &mut [f32]) {
    let n: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if n > 0.0 {
        for x in v.iter_mut() {
            *x /= n;
        }
    }
}

/// Download the MiniLM model + tokenizer from HuggingFace Hub into
/// `<cache_dir>/all-MiniLM-L6-v2/`. Returns the destination directory.
///
/// This is the production entry point; tests should use
/// [`onq_test_utils::mock_ort::MockEmbedder`] instead so they
/// don't hit the network.
pub async fn download_model(cache_dir: &Path) -> CoreResult<PathBuf> {
    let api = hf_hub::api::tokio::Api::new().map_err(|e| CoreError::Other(e.to_string()))?;
    let repo = api.model(MODEL_ID.to_string());
    let onnx = repo
        .get("onnx/model.onnx")
        .await
        .map_err(|e| CoreError::Other(e.to_string()))?;
    let tok = repo
        .get("tokenizer.json")
        .await
        .map_err(|e| CoreError::Other(e.to_string()))?;
    let dst = cache_dir.join("all-MiniLM-L6-v2");
    tokio::fs::create_dir_all(&dst)
        .await
        .map_err(CoreError::Io)?;
    tokio::fs::copy(&onnx, dst.join("model.onnx"))
        .await
        .map_err(CoreError::Io)?;
    tokio::fs::copy(&tok, dst.join("tokenizer.json"))
        .await
        .map_err(CoreError::Io)?;
    Ok(dst)
}

#[cfg(test)]
mod tests {
    use super::{l2_normalize, mean_pool};
    use onq_test_utils::mock_ort::MockEmbedder;

    #[test]
    fn mock_returns_384d_unit_vector() {
        let m = MockEmbedder::new("test");
        let v = m.embed();
        assert_eq!(v.len(), 384);
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-3);
    }

    #[test]
    fn mean_pool_averages_real_tokens_and_ignores_padding() {
        // 5 tokens x 3 dims, mask = [1, 1, 1, 0, 0].
        let data = vec![
            1.0, 2.0, 3.0, // token 0
            4.0, 5.0, 6.0, // token 1
            7.0, 8.0, 9.0, // token 2
            10.0, 11.0, 12.0, // padding
            13.0, 14.0, 15.0, // padding
        ];
        let mask = vec![1i64, 1, 1, 0, 0];
        let pooled = mean_pool(&data, &mask, 3);
        assert_eq!(pooled.len(), 3);
        // (1+4+7)/3 = 4, (2+5+8)/3 = 5, (3+6+9)/3 = 6
        assert!((pooled[0] - 4.0).abs() < 1e-6);
        assert!((pooled[1] - 5.0).abs() < 1e-6);
        assert!((pooled[2] - 6.0).abs() < 1e-6);
    }

    #[test]
    fn mean_pool_with_all_padding_returns_zeros() {
        // Avoids div-by-zero: real_count == 0 leaves sum untouched (all zero).
        let data = vec![1.0, 2.0, 3.0];
        let mask = vec![0i64];
        let pooled = mean_pool(&data, &mask, 3);
        assert_eq!(pooled, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn l2_normalize_scales_to_unit_norm() {
        let mut v = vec![3.0f32, 4.0]; // norm = 5
        l2_normalize(&mut v);
        let n: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((n - 1.0).abs() < 1e-6);
        assert!((v[0] - 0.6).abs() < 1e-6);
        assert!((v[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn l2_normalize_zero_vector_is_noop() {
        let mut v = vec![0.0f32, 0.0, 0.0];
        l2_normalize(&mut v);
        assert_eq!(v, vec![0.0, 0.0, 0.0]);
    }
}
