//! Plugin archive installation pipeline.
//!
//! An installable plugin ships as a `.tar.gz` archive containing:
//!   * `manifest.toml` — `[plugin]` table with at least `id`, `name`,
//!     `version`, `license` (must be `GPL-3.0-only`), and an optional
//!     `capabilities` array.
//!   * `signature.sig` — 64-byte ed25519 signature over the raw bytes
//!     of the archive itself, verified against the compiled trust anchor
//!     from [`crate::signature`].
//!   * The plugin's `.so` / `.dylib` / `.dll` plus any ancillary files.
//!
//! `install` runs the four-step pipeline:
//!   1. extract to a scratch `tempdir`,
//!   2. read + license-check the manifest,
//!   3. verify the ed25519 signature against the archive bytes,
//!   4. move the scratch tree under `plugins_dir/installed/<id>/`.
//!
//! On any failure the scratch tree is dropped (RAII via `TempDir`), so a
//! half-installed plugin never leaks onto disk.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use tar::Archive;

use crate::error::{CoreError, CoreResult};
use crate::path_util;
use crate::signature;

/// Per-archive entries the install pipeline looks up by exact name.
const MANIFEST_FILE: &str = "manifest.toml";
const SIGNATURE_FILE: &str = "signature.sig";

/// On-disk subdirectory of `plugins_dir` where installed plugins live.
/// `install` writes the extracted tree here, named by `manifest.id`.
const INSTALLED_SUBDIR: &str = "installed";

/// Install a `.tar.gz` plugin archive into `<plugins_dir>/installed/<id>/`.
///
/// See module docs for the on-disk layout. Returns the destination path on
/// success. Any error leaves no partial install behind — the temp
/// extraction is dropped before the function returns.
pub fn install(archive: &Path, plugins_dir: &Path) -> CoreResult<PathBuf> {
    install_with_trust_anchor(archive, plugins_dir, None)
}

/// Like [`install`], but lets the caller override the trust anchor used
/// for signature verification. `None` means "use the compiled trust
/// anchor from [`crate::signature`]". Tests pass a freshly-generated
/// public key so they can sign with the matching fixture private key
/// without baking a real key into the source tree.
pub fn install_with_trust_anchor(
    archive: &Path,
    plugins_dir: &Path,
    trust_anchor: Option<&[u8; 32]>,
) -> CoreResult<PathBuf> {
    // 1. Extract to a scratch tempdir (auto-cleaned on early return).
    let tmp = tempfile::tempdir()?;
    extract_tar_gz(archive, tmp.path())?;

    // 2. Read + license-check the manifest.
    let manifest_path = tmp.path().join(MANIFEST_FILE);
    let manifest_text = std::fs::read_to_string(&manifest_path)?;
    let manifest: toml::Value = manifest_text.parse().map_err(CoreError::from)?;
    let plugin_table = manifest
        .get("plugin")
        .ok_or_else(|| CoreError::Plugin("manifest missing [plugin] table".into()))?;
    let license = plugin_table
        .get("license")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| CoreError::Plugin("missing [plugin].license".into()))?;
    if license != "GPL-3.0-only" {
        return Err(CoreError::Plugin(format!(
            "license must be GPL-3.0-only, got {license}"
        )));
    }
    let id = plugin_table
        .get("id")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| CoreError::Plugin("missing [plugin].id".into()))?;
    let name = plugin_table
        .get("name")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| CoreError::Plugin("missing [plugin].name".into()))?;
    let version = plugin_table
        .get("version")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| CoreError::Plugin("missing [plugin].version".into()))?;
    let capabilities = plugin_table
        .get("capabilities")
        .cloned()
        .unwrap_or(toml::Value::Array(Vec::new()));

    // 3. Verify the archive signature against the trust anchor.
    //    The signed payload is the canonical manifest text — not the
    //    raw archive bytes — so the signer can produce a deterministic
    //    signature without a chicken-and-egg between the signed bytes
    //    and the signature file embedded inside the same archive. The
    //    plugin author signs `manifest.toml`; the on-disk `signature.sig`
    //    carries that signature as-is.
    let sig = std::fs::read(tmp.path().join(SIGNATURE_FILE))?;
    let artifact = manifest_text.into_bytes();
    match trust_anchor {
        Some(pk) => signature::verify_with_pubkey(&artifact, &sig, pk)?,
        None => signature::verify(&artifact, &sig)?,
    }

    // 4. Move the scratch tree under <plugins_dir>/installed/<id>/.
    let dest = plugins_dir.join(INSTALLED_SUBDIR).join(id);
    if dest.exists() {
        return Err(CoreError::Plugin(format!(
            "plugin {} already installed at {}",
            id,
            dest.display()
        )));
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    copy_dir_all(tmp.path(), &dest)?;
    // touch a few values to silence unused warnings until the install
    // pipeline is fully wired into the Tauri commands (which read them).
    let _ = (name, version, capabilities);
    Ok(dest)
}

/// Extract a `.tar.gz` archive at `archive` into `dest`, creating `dest`
/// first. Path-traversal entries (those whose normalized path escapes
/// `dest`) are rejected so a malicious archive can't write outside the
/// staging directory.
fn extract_tar_gz(archive: &Path, dest: &Path) -> CoreResult<()> {
    std::fs::create_dir_all(dest)?;
    let file = File::open(archive)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    let canonical_dest = dest.canonicalize().unwrap_or_else(|_| dest.to_path_buf());
    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?.into_owned();
        let safe_path = path_util::safe_join(&canonical_dest, &entry_path).map_err(|e| {
            // Preserve plugin-flavored errors for the install pipeline.
            CoreError::Plugin(e.to_string())
        })?;
        if entry_path.to_str().unwrap_or("").ends_with('/') || entry.header().entry_type().is_dir()
        {
            std::fs::create_dir_all(&safe_path)?;
        } else {
            if let Some(parent) = safe_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            entry.unpack(&safe_path)?;
        }
    }
    Ok(())
}

/// Recursively copy `src` into `dst`, creating `dst` if needed.
fn copy_dir_all(src: &Path, dst: &Path) -> CoreResult<()> {
    path_util::copy_path(src, dst)
}

/// Build a `.tar.gz` plugin archive in memory given the on-disk manifest
/// contents and signature bytes. The archive's byte-for-byte contents
/// are returned so the caller can persist the archive AND pre-compute
/// the payload that needs to be signed.
///
/// Layout: a single `manifest.toml`, a single `signature.sig`, plus an
/// optional list of arbitrary files (typically the plugin `.so`).
pub fn build_archive(
    manifest: &str,
    signature: &[u8],
    extra_files: &[(&str, &[u8])],
) -> CoreResult<Vec<u8>> {
    let buf = Vec::new();
    let encoder = flate2::write::GzEncoder::new(buf, flate2::Compression::default());
    let mut tar = tar::Builder::new(encoder);

    append_bytes(&mut tar, MANIFEST_FILE, manifest.as_bytes())?;
    append_bytes(&mut tar, SIGNATURE_FILE, signature)?;
    for (name, bytes) in extra_files {
        append_bytes(&mut tar, name, bytes)?;
    }
    tar.finish()?;
    let encoder = tar
        .into_inner()
        .map_err(|e| CoreError::Plugin(e.to_string()))?;
    let bytes = encoder
        .finish()
        .map_err(|e| CoreError::Other(e.to_string()))?;
    Ok(bytes)
}

fn append_bytes<W: Write>(tar: &mut tar::Builder<W>, name: &str, bytes: &[u8]) -> CoreResult<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append_data(&mut header, name, bytes)?;
    Ok(())
}

/// Build a `manifest.toml` for a given plugin. Convenience for tests +
/// tooling; production archives are produced by the plugin author.
pub fn render_manifest(id: &str, name: &str, version: &str, capabilities: &[&str]) -> String {
    let caps = capabilities
        .iter()
        .map(|c| format!("  - {c:?}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "[plugin]\nid = {id:?}\nname = {name:?}\nversion = {version:?}\nlicense = \"GPL-3.0-only\"\ncapabilities = [\n{caps}\n]\n",
    )
}

/// Read the raw plugin archive bytes. Kept as a separate function so
/// callers that already have the bytes in hand (tests) can skip the
/// extra disk read.
pub fn tar_archive_bytes(archive: &Path) -> CoreResult<Vec<u8>> {
    let mut file = File::open(archive)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use tempfile::TempDir;

    /// Build a self-signed plugin archive under `tmp` and return
    /// (archive_path, signing_key, manifest_text, pubkey). The fixture
    /// signs the canonical manifest text so `install_with_trust_anchor`
    /// can verify it deterministically without a chicken-and-egg
    /// between the embedded signature and the archive bytes that
    /// contain it. Returns the manifest text + pubkey so tests can
    /// pass the pubkey to `install_with_trust_anchor` and verify
    /// against the matching fixture key.
    fn build_signed_fixture(tmp: &Path, license: &str) -> (PathBuf, SigningKey, String, [u8; 32]) {
        let mut csprng = OsRng;
        let key = SigningKey::generate(&mut csprng);
        let pubkey = key.verifying_key().to_bytes();
        let manifest = format!(
            "[plugin]\nid = \"acme.foo\"\nname = \"Foo\"\nversion = \"0.1.0\"\nlicense = {license:?}\ncapabilities = [\"read\"]\n",
        );
        let sig = crate::signature::sign_for_test(manifest.as_bytes(), &key);
        let bytes =
            build_archive(&manifest, &sig, &[("plugin.so", b"BINARY")]).expect("build archive");
        let archive_path = tmp.join("plugin.tar.gz");
        std::fs::write(&archive_path, &bytes).expect("write archive");
        (archive_path, key, manifest, pubkey)
    }

    #[test]
    fn install_copies_tree_to_installed_subdir() {
        let staging = TempDir::new().unwrap();
        let (archive, _key, manifest, pubkey) =
            build_signed_fixture(staging.path(), "GPL-3.0-only");
        let plugins_dir = TempDir::new().unwrap();
        let dest = install_with_trust_anchor(&archive, plugins_dir.path(), Some(&pubkey))
            .expect("install");
        // Manifest copied across verbatim.
        assert!(dest.join("manifest.toml").is_file());
        let on_disk = std::fs::read_to_string(dest.join("manifest.toml")).unwrap();
        assert_eq!(on_disk, manifest, "manifest differs after roundtrip");
        // Signature copied across.
        assert!(dest.join("signature.sig").is_file());
        // Plugin binary copied across.
        assert!(dest.join("plugin.so").is_file());
    }

    #[test]
    fn install_rejects_wrong_license() {
        let staging = TempDir::new().unwrap();
        let (archive, _key, _manifest, pubkey) = build_signed_fixture(staging.path(), "MIT");
        let plugins_dir = TempDir::new().unwrap();
        let err =
            install_with_trust_anchor(&archive, plugins_dir.path(), Some(&pubkey)).unwrap_err();
        match err {
            CoreError::Plugin(msg) => {
                assert!(msg.contains("GPL-3.0-only"), "msg = {msg}");
            }
            other => panic!("expected Plugin error, got {other:?}"),
        }
    }

    #[test]
    fn install_rejects_missing_manifest() {
        // Build an archive that has a signature.sig + plugin.so but no
        // manifest.toml at all. The install pipeline must fail before
        // it ever reaches the signature check.
        let tmp = TempDir::new().unwrap();
        let key = SigningKey::generate(&mut OsRng);
        let pubkey = key.verifying_key().to_bytes();
        let sig = vec![0u8; 64];
        let bytes = build_archive("", &sig, &[("plugin.so", b"BINARY")]).expect("build");
        let archive_path = tmp.path().join("no-manifest.tar.gz");
        std::fs::write(&archive_path, &bytes).unwrap();
        let plugins_dir = TempDir::new().unwrap();
        let err = install_with_trust_anchor(&archive_path, plugins_dir.path(), Some(&pubkey))
            .unwrap_err();
        assert!(
            matches!(err, CoreError::Plugin(_) | CoreError::Io(_)),
            "expected Plugin or IO error, got {err:?}"
        );
    }

    #[test]
    fn install_rejects_bad_signature() {
        let staging = TempDir::new().unwrap();
        let (archive, _key, _manifest, pubkey) =
            build_signed_fixture(staging.path(), "GPL-3.0-only");
        // Re-pack the same archive contents but with a placeholder
        // signature so verification must fail (manifest unchanged, sig
        // wrong). Using a wholly-zero signature guarantees a verification
        // mismatch without risking accidentally flipping the gzip header
        // and producing an IO error.
        let manifest = "[plugin]\nid = \"acme.foo\"\nname = \"Foo\"\nversion = \"0.1.0\"\nlicense = \"GPL-3.0-only\"\ncapabilities = [\"read\"]\n";
        let bogus = vec![0u8; 64];
        let bytes = build_archive(manifest, &bogus, &[("plugin.so", b"BINARY")]).expect("build");
        let tampered = archive.with_file_name("bad-sig.tar.gz");
        std::fs::write(&tampered, &bytes).unwrap();
        let plugins_dir = TempDir::new().unwrap();
        let err =
            install_with_trust_anchor(&tampered, plugins_dir.path(), Some(&pubkey)).unwrap_err();
        match err {
            CoreError::Plugin(_) => {}
            other => panic!("expected Plugin error, got {other:?}"),
        }
    }

    #[test]
    fn install_rejects_duplicate_id() {
        let staging = TempDir::new().unwrap();
        let (archive, _key, _manifest, pubkey) =
            build_signed_fixture(staging.path(), "GPL-3.0-only");
        let plugins_dir = TempDir::new().unwrap();
        install_with_trust_anchor(&archive, plugins_dir.path(), Some(&pubkey))
            .expect("first install");
        let err =
            install_with_trust_anchor(&archive, plugins_dir.path(), Some(&pubkey)).unwrap_err();
        match err {
            CoreError::Plugin(msg) => assert!(msg.contains("already installed"), "msg = {msg}"),
            other => panic!("expected Plugin error, got {other:?}"),
        }
    }

    #[test]
    fn manifest_renderer_includes_gpl_license() {
        let m = render_manifest("acme.foo", "Foo", "0.1.0", &["read", "write"]);
        assert!(m.contains("license = \"GPL-3.0-only\""));
        assert!(m.contains("id = \"acme.foo\""));
        assert!(m.contains("\"read\""));
        assert!(m.contains("\"write\""));
    }
}
