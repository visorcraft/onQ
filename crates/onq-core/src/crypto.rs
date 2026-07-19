//! Per-prompt body encryption (AES-256-GCM).
//!
//! Used by the lock flow (M5.2): each locked prompt has its own randomly
//! generated 32-byte data-encryption key (DEK), stored in the OS keychain.
//! The DEK is the symmetric key here; the master passphrase's KEK only
//! wraps the vault-level database, never per-prompt bodies.
//!
//! The on-disk envelope is a small framed container:
//!
//! ```text
//! +--------+----------------+--------------------+----------+
//! | ver(1) | nonce (12 B)   | ciphertext (N B)   | tag (16B)|
//! +--------+----------------+--------------------+----------+
//! ```
//!
//! - `ver` — currently always `1`. Bumping it lets us migrate the format
//!   without breaking existing `.enc` files.
//! - `nonce` — fresh 96-bit random value per encrypt. Required by AES-GCM;
//!   never reused with the same key.
//! - `ciphertext + tag` — the value returned by `aes-gcm`'s `encrypt`/`decrypt`
//!   calls (the crate appends the 16-byte GHASH authentication tag to the plaintext
//!   length and bakes it onto the ciphertext).

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::RngCore;

use crate::error::{CoreError, CoreResult};

/// Current envelope version. Bumped on incompatible format changes.
const ENVELOPE_VERSION: u8 = 1;
const NONCE_LEN: usize = 12;

/// Encrypt `plaintext` under `key` and return the framed envelope.
///
/// A fresh 96-bit nonce is drawn from the OS CSPRNG (`rand::thread_rng`)
/// for every call. The same `(key, nonce)` pair must never be reused.
pub fn encrypt_body(key: &[u8; 32], plaintext: &[u8]) -> CoreResult<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // aes-gcm appends the 16-byte auth tag to the ciphertext itself.
    let ct = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CoreError::Encryption(format!("aes-gcm encrypt: {e}")))?;

    let mut out = Vec::with_capacity(1 + NONCE_LEN + ct.len());
    out.push(ENVELOPE_VERSION);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

/// Decrypt a previously produced envelope under `key`. Returns
/// [`CoreError::Encryption`] on a malformed envelope, wrong version,
/// bad key, or any ciphertext/tag tampering.
pub fn decrypt_body(key: &[u8; 32], envelope: &[u8]) -> CoreResult<Vec<u8>> {
    if envelope.is_empty() {
        return Err(CoreError::Encryption("envelope is empty".into()));
    }
    if envelope[0] != ENVELOPE_VERSION {
        return Err(CoreError::Encryption(format!(
            "unsupported envelope version: {}",
            envelope[0]
        )));
    }
    if envelope.len() < 1 + NONCE_LEN {
        return Err(CoreError::Encryption("envelope too short for nonce".into()));
    }

    let nonce = Nonce::from_slice(&envelope[1..1 + NONCE_LEN]);
    let ct = &envelope[1 + NONCE_LEN..];

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    cipher
        .decrypt(nonce, ct)
        .map_err(|e| CoreError::Encryption(format!("aes-gcm decrypt: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let k = key(0xAA);
        let plaintext = b"the quick brown fox jumps over the lazy dog";

        let env = encrypt_body(&k, plaintext).expect("encrypt ok");
        // version(1) + nonce(12) + ciphertext(N) + tag(16)
        assert_eq!(env[0], ENVELOPE_VERSION);
        assert!(env.len() >= 1 + NONCE_LEN + plaintext.len() + 16);

        let pt = decrypt_body(&k, &env).expect("decrypt ok");
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn decrypt_wrong_key_fails() {
        let env = encrypt_body(&key(1), b"secret body").expect("encrypt ok");

        let mut wrong = key(1);
        wrong[0] ^= 0x01; // flip a single bit

        let err = decrypt_body(&wrong, &env).expect_err("wrong key must fail");
        assert!(
            matches!(err, CoreError::Encryption(_)),
            "expected CoreError::Encryption, got {err:?}"
        );
    }

    #[test]
    fn decrypt_tampered_ciphertext_fails() {
        let k = key(7);
        let mut env = encrypt_body(&k, b"tamper me").expect("encrypt ok");

        // Flip a bit somewhere in the ciphertext+tag region (past the nonce).
        let idx = 1 + NONCE_LEN + 2;
        env[idx] ^= 0x80;

        let err = decrypt_body(&k, &env).expect_err("tampered envelope must fail");
        assert!(
            matches!(err, CoreError::Encryption(_)),
            "expected CoreError::Encryption, got {err:?}"
        );
    }
}
