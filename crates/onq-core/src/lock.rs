//! Master passphrase + Argon2id KDF.
//!
//! The master passphrase gates access to the encrypted search index DB.
//! It is generated once at vault creation, stored in the OS keychain via
//! [`crate::keychain::Keychain`], and used to derive a per-vault KEK
//! (key-encryption key) that unlocks the DB. Argon2id parameters follow
//! the OWASP Password Storage Cheat Sheet recommendations for interactive
//! logins (m=64 MiB, t=3, p=4, output=32 B).

use argon2::{Algorithm, Argon2, Params, Version};
use rand::distributions::Alphanumeric;
use rand::{Rng, RngCore};

use crate::error::{CoreError, CoreResult};

/// Derive a 32-byte KEK from a master passphrase and a per-vault salt.
///
/// Uses Argon2id (m=64 MiB, t=3, p=4, output_len=32). Returns
/// [`CoreError::Encryption`] on parameter validation or KDF failure.
pub fn derive_kek(passphrase: &[u8], salt: &[u8; 32]) -> CoreResult<[u8; 32]> {
    // OWASP Password Storage Cheat Sheet (interactive login tier): m=64 MiB,
    // t=3, p=4. 32-byte output fits a chacha20-poly1305 key without padding.
    let params = Params::new(64 * 1024, 3, 4, Some(32))
        .map_err(|e| CoreError::Encryption(format!("argon2 params: {e}")))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = [0u8; 32];
    argon
        .hash_password_into(passphrase, salt, &mut out)
        .map_err(|e| CoreError::Encryption(format!("argon2 hash: {e}")))?;
    Ok(out)
}

/// Generate a fresh 32-byte salt from the OS CSPRNG.
pub fn generate_salt() -> [u8; 32] {
    let mut s = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut s);
    s
}

/// Generate a 64-character random alphanumeric master passphrase.
///
/// Returned ONCE at vault setup and shown to the user as the recovery
/// phrase UI; thereafter it is fetched from the OS keychain.
pub fn generate_passphrase() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kek_deterministic() {
        let salt = [7u8; 32];
        let a = derive_kek(b"hunter2", &salt).unwrap();
        let b = derive_kek(b"hunter2", &salt).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn kek_differs_for_salt() {
        let a = derive_kek(b"hunter2", &[1u8; 32]).unwrap();
        let b = derive_kek(b"hunter2", &[2u8; 32]).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn generate_salt_is_32_bytes_and_random() {
        let a = generate_salt();
        let b = generate_salt();
        assert_eq!(a.len(), 32);
        assert_ne!(a, b, "two consecutive salts must differ");
    }

    #[test]
    fn generate_passphrase_is_64_alphanumeric() {
        let p = generate_passphrase();
        assert_eq!(p.len(), 64);
        assert!(
            p.chars().all(|c| c.is_ascii_alphanumeric()),
            "passphrase must be ASCII alphanumeric, got {p:?}"
        );
    }
}
