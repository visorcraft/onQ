use ed25519_dalek::{Signature, Verifier, VerifyingKey};

#[cfg(test)]
use ed25519_dalek::{Signer, SigningKey};

use crate::error::{CoreError, CoreResult};

// Public key compiled into binary (dev key for now; rotate before release).
const TRUST_ANCHOR_PUB: [u8; 32] = *include_bytes!("../trust_anchor.pub");

pub fn verify(artifact: &[u8], signature: &[u8]) -> CoreResult<()> {
    verify_with_pubkey(artifact, signature, &TRUST_ANCHOR_PUB)
}

/// Verify an artifact signature against an explicit public key. Production
/// callers go through [`verify`] (which uses the compiled trust anchor);
/// tests use this seam so they can sign with a fixture key and verify
/// against the matching fixture pubkey without baking a private key into
/// the source tree.
pub fn verify_with_pubkey(artifact: &[u8], signature: &[u8], pubkey: &[u8; 32]) -> CoreResult<()> {
    let key =
        VerifyingKey::from_bytes(pubkey).map_err(|error| CoreError::Plugin(error.to_string()))?;
    let signature =
        Signature::from_slice(signature).map_err(|error| CoreError::Plugin(error.to_string()))?;

    key.verify(artifact, &signature)
        .map_err(|error| CoreError::Plugin(error.to_string()))
}

#[cfg(test)]
pub fn sign_for_test(artifact: &[u8], key: &SigningKey) -> Vec<u8> {
    key.sign(artifact).to_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRUSTED_SIGNATURE: [u8; 64] = [
        0xf4, 0x4b, 0xda, 0xcf, 0x88, 0x26, 0x46, 0x8a, 0xf4, 0xe7, 0x72, 0x36, 0xb4, 0xcb, 0x37,
        0xbd, 0xe3, 0x3d, 0x72, 0x5f, 0x40, 0x63, 0x53, 0x0c, 0x21, 0xe7, 0x9d, 0x35, 0x95, 0x8a,
        0x34, 0x4e, 0x01, 0xc5, 0x60, 0xb4, 0x92, 0x60, 0xd2, 0xfd, 0x8a, 0x88, 0x29, 0x59, 0x1e,
        0x26, 0x2f, 0x59, 0x55, 0xff, 0x70, 0xc9, 0x77, 0x8f, 0xb2, 0x63, 0x0f, 0x22, 0x7a, 0xbf,
        0x36, 0x4f, 0x03, 0x06,
    ];

    #[test]
    fn roundtrip() {
        verify(b"hello", &TRUSTED_SIGNATURE).unwrap();
    }

    #[test]
    fn rejects_tampered() {
        let result = verify(b"different", &TRUSTED_SIGNATURE);

        assert!(result.is_err());
    }
}
