//! Versioned `.onqbak` container format.
//!
//! Layout (v1):
//! ```text
//! magic[8] = b"ONQBAK\x01\x00"
//! flags[1]  = 0 plain | 1 sealed
//! if plain:   gzipped-tar payload…
//! if sealed:  argon2-salt[32] || aes-gcm envelope(gzipped-tar)
//! ```
//!
//! The payload bytes are opaque to this module — see [`super::payload`].
//! Adding a new seal scheme means a new `flags` value + branch here without
//! touching pack/unpack or import replace logic.

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::crypto;
use crate::error::{CoreError, CoreResult};
use crate::lock::{derive_kek, generate_salt};

/// On-disk magic + version (`ONQBAK` + major `1` + minor `0`).
pub const MAGIC: &[u8; 8] = b"ONQBAK\x01\x00";

/// Unencrypted payload follows the header.
pub const FLAG_PLAIN: u8 = 0;
/// Argon2id + AES-256-GCM sealed payload.
pub const FLAG_SEALED: u8 = 1;

/// How the gzipped-tar payload is protected inside the container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealMode<'a> {
    /// Store payload as-is (vault contents remain vault-encrypted on disk).
    Plain,
    /// Outer password independent of the vault KEK / master password.
    Password(&'a str),
}

impl<'a> SealMode<'a> {
    /// Treat missing/blank passwords as plain (UI "optional password").
    pub fn from_optional_password(password: Option<&'a str>) -> Self {
        match password.map(str::trim).filter(|p| !p.is_empty()) {
            Some(p) => SealMode::Password(p),
            None => SealMode::Plain,
        }
    }
}

/// Write a complete `.onqbak` file for the given payload bytes.
pub fn write_container(dest: &Path, payload: &[u8], seal: SealMode<'_>) -> CoreResult<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut out = File::create(dest)?;
    out.write_all(MAGIC)?;

    match seal {
        SealMode::Plain => {
            out.write_all(&[FLAG_PLAIN])?;
            out.write_all(payload)?;
        }
        SealMode::Password(pw) => {
            let salt = generate_salt();
            let key = derive_kek(pw.as_bytes(), &salt)?;
            let envelope = crypto::encrypt_body(&key, payload)?;
            out.write_all(&[FLAG_SEALED])?;
            out.write_all(&salt)?;
            out.write_all(&envelope)?;
        }
    }
    out.flush()?;
    Ok(())
}

/// Read and (if needed) unseal the payload from a `.onqbak` file.
pub fn read_container(archive: &Path, password: Option<&str>) -> CoreResult<Vec<u8>> {
    let mut file = File::open(archive)?;
    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)
        .map_err(|_| CoreError::Other("backup file is truncated or empty".into()))?;
    if &magic != MAGIC {
        return Err(CoreError::Other(
            "not an onQ backup (.onqbak) or unsupported version".into(),
        ));
    }

    let mut flag = [0u8; 1];
    file.read_exact(&mut flag)
        .map_err(|_| CoreError::Other("backup file is truncated".into()))?;

    match flag[0] {
        FLAG_PLAIN => {
            let mut payload = Vec::new();
            file.read_to_end(&mut payload)?;
            Ok(payload)
        }
        FLAG_SEALED => {
            let pw = password
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .ok_or_else(|| CoreError::Other("this backup is password-protected".into()))?;
            let mut salt = [0u8; 32];
            file.read_exact(&mut salt)
                .map_err(|_| CoreError::Other("backup file is truncated".into()))?;
            let mut envelope = Vec::new();
            file.read_to_end(&mut envelope)?;
            let key = derive_kek(pw.as_bytes(), &salt)?;
            crypto::decrypt_body(&key, &envelope)
                .map_err(|_| CoreError::Other("wrong archive password or corrupted backup".into()))
        }
        other => Err(CoreError::Other(format!(
            "unknown backup seal flag: {other:#x} (upgrade onQ?)"
        ))),
    }
}

/// Probe whether an on-disk file is a sealed (password-protected) archive
/// without requiring the password. Useful for UI to prompt before import.
pub fn is_sealed_archive(archive: &Path) -> CoreResult<bool> {
    let mut file = File::open(archive)?;
    let mut header = [0u8; 9];
    file.read_exact(&mut header)
        .map_err(|_| CoreError::Other("backup file is truncated or empty".into()))?;
    if &header[..8] != MAGIC {
        return Err(CoreError::Other(
            "not an onQ backup (.onqbak) or unsupported version".into(),
        ));
    }
    Ok(header[8] == FLAG_SEALED)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn plain_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a.onqbak");
        write_container(&path, b"hello-payload", SealMode::Plain).unwrap();
        assert!(!is_sealed_archive(&path).unwrap());
        assert_eq!(read_container(&path, None).unwrap(), b"hello-payload");
    }

    #[test]
    fn sealed_requires_password() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("s.onqbak");
        write_container(&path, b"secret", SealMode::Password("pw")).unwrap();
        assert!(is_sealed_archive(&path).unwrap());
        assert!(read_container(&path, None)
            .unwrap_err()
            .to_string()
            .contains("password"));
        assert!(read_container(&path, Some("nope")).is_err());
        assert_eq!(read_container(&path, Some("pw")).unwrap(), b"secret");
    }

    #[test]
    fn blank_password_is_plain() {
        assert_eq!(
            SealMode::from_optional_password(Some("  ")),
            SealMode::Plain
        );
        assert_eq!(
            SealMode::from_optional_password(Some("x")),
            SealMode::Password("x")
        );
    }
}
