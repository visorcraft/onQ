//! Plugin archive installation pipeline.
//!
//! An installable plugin ships as a `.tar.gz` archive containing:
//!   * `manifest.toml`: `[plugin]` table with at least `id`, `name`,
//!     `version`, `license` (must be `GPL-3.0-only`), and an optional
//!     `capabilities` array.
//!   * `signature.sig`: 64-byte ed25519 signature over the signed payload,
//!     verified against the compiled trust anchor from [`crate::signature`].
//!   * The plugin's `.so` / `.dylib` / `.dll` plus any ancillary files.
//!
//! The signed payload is the exact `manifest.toml` bytes, followed by a
//! newline, followed by one line per regular file in the extracted staging
//! tree (excluding `signature.sig`): `<lowercase-hex sha256>  <relative
//! path>\n`, sorted by relative path bytes. Hashing every archive file
//! means a tampered library, an added file, or a swapped manifest all
//! break verification, so the signature covers the whole archive contents,
//! not just the manifest text.
//!
//! `install` runs the four-step pipeline:
//!   1. extract to a scratch `tempdir`,
//!   2. read + license-check the manifest,
//!   3. verify the ed25519 signature against the signed payload,
//!   4. copy the scratch tree under `plugins_dir/installed/<id>/`.
//!
//! On any failure the scratch tree is dropped (RAII via `TempDir`), so a
//! half-installed plugin never leaks onto disk.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
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

/// Bounds on pre-verification extraction of an untrusted archive. A gzip
/// "tar bomb" (kilobytes compressed -> gigabytes decompressed, or millions
/// of entries) must be rejected before the signature is verified, so the
/// disk / inode table is never filled by an unsigned archive. These are
/// bomb stoppers, not a plugin-size policy.
const MAX_EXTRACTED_BYTES: u64 = 1 << 30;
const MAX_EXTRACTED_ENTRIES: usize = 10_000;

/// Message used when a decompressed stream exceeds the byte cap.
const SIZE_LIMIT_MSG: &str = "plugin archive exceeds the decompressed size limit";

/// A reader that caps the decompressed byte stream flowing through it.
///
/// The cap applies to decompressed bytes (it wraps the `GzDecoder`), not to
/// the compressed file size. A clean EOF exactly at the limit is fine; any
/// further byte fails with `InvalidData` instead of reporting EOF.
struct BoundedReader<R> {
    inner: R,
    remaining: u64,
}

impl<R: std::io::Read> std::io::Read for BoundedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.remaining == 0 {
            // A clean EOF exactly at the limit is fine; any further byte is not.
            let mut probe = [0u8; 1];
            return match self.inner.read(&mut probe)? {
                0 => Ok(0),
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    SIZE_LIMIT_MSG,
                )),
            };
        }
        let want = buf.len().min(self.remaining as usize);
        let n = self.inner.read(&mut buf[..want])?;
        self.remaining -= n as u64;
        Ok(n)
    }
}

/// Lowercase hex sha256 of `bytes` (64 chars).
fn sha256_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        hex.push(HEX[(byte >> 4) as usize] as char);
        hex.push(HEX[(byte & 0x0f) as usize] as char);
    }
    hex
}

/// Validate one archive-relative path for use in a digest line.
///
/// Returns the `/`-separated relative path on success. Rejects with
/// [`CoreError::Plugin`]:
///   * any component that is not valid UTF-8,
///   * any component containing a backslash,
///   * a file named `signature.sig` anywhere other than the archive root
///     (the root `signature.sig` is excluded from hashing, but a second
///     `signature.sig` nested in the tree is ambiguous and rejected),
///   * paths with `..`, roots, or prefixes (only normal relative
///     components are allowed).
fn validate_rel_path(raw: &Path) -> CoreResult<String> {
    let mut components: Vec<String> = Vec::new();
    for comp in raw.components() {
        match comp {
            Component::Normal(name) => {
                let name = name.to_str().ok_or_else(|| {
                    CoreError::Plugin(format!("non-UTF-8 path in staging tree: {}", raw.display()))
                })?;
                if name.contains('\\') {
                    return Err(CoreError::Plugin(format!(
                        "backslash in relative path: {}",
                        raw.display()
                    )));
                }
                components.push(name.to_string());
            }
            Component::CurDir => {}
            _ => {
                return Err(CoreError::Plugin(format!(
                    "invalid relative path: {}",
                    raw.display()
                )))
            }
        }
    }
    let Some(basename) = components.last() else {
        return Err(CoreError::Plugin(format!(
            "empty relative path: {}",
            raw.display()
        )));
    };
    if components.len() > 1 && basename == SIGNATURE_FILE {
        return Err(CoreError::Plugin(format!(
            "signature.sig found outside the archive root: {}",
            raw.display()
        )));
    }
    Ok(components.join("/"))
}

/// Build the deterministic signed payload for a staging tree: the exact
/// manifest bytes, then a newline, then one line per regular file (excluding
/// `signature.sig`): `<lowercase-hex sha256>  <relative path>\n`, sorted by
/// relative path bytes.
///
/// `files` carries the archive-relative path plus file bytes; callers read
/// the bytes themselves (the install pipeline from the extracted tree, the
/// author-side builder from in-memory extra files). Pathological shapes are
/// rejected by [`validate_rel_path`] before any hashing happens.
fn payload_from_files(manifest_text: &str, files: &[(PathBuf, Vec<u8>)]) -> CoreResult<Vec<u8>> {
    let mut validated: Vec<(String, Vec<u8>)> = Vec::with_capacity(files.len());
    for (rel, bytes) in files {
        let rel = validate_rel_path(rel)?;
        if rel == SIGNATURE_FILE {
            continue;
        }
        validated.push((rel, bytes.clone()));
    }
    validated.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

    let mut payload = Vec::new();
    payload.extend_from_slice(manifest_text.as_bytes());
    payload.push(b'\n');
    for (rel, bytes) in &validated {
        let hex = sha256_hex(bytes);
        payload.extend_from_slice(hex.as_bytes());
        payload.extend_from_slice(b"  ");
        payload.extend_from_slice(rel.as_bytes());
        payload.push(b'\n');
    }
    Ok(payload)
}

/// Walk `dir` recursively and push every regular file as
/// `(archive-relative path, absolute path)` into `out`.
///
/// Symlinks and other non-regular entries are rejected with
/// [`CoreError::Plugin`]. Note that `extract_tar_gz` materializes symlink
/// entries into the staging tree via `entry.unpack()`, so an archive can
/// plant one — it must not pass verification unhashed.
fn collect_regular_files(
    dir: &Path,
    rel_parts: &mut Vec<std::ffi::OsString>,
    out: &mut Vec<(PathBuf, PathBuf)>,
) -> CoreResult<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        rel_parts.push(entry.file_name());
        let rel = rel_parts.iter().collect::<PathBuf>();
        if file_type.is_dir() {
            collect_regular_files(&path, rel_parts, out)?;
        } else if file_type.is_file() {
            out.push((rel, path));
        } else {
            rel_parts.pop();
            return Err(CoreError::Plugin(format!(
                "unsupported non-regular entry in staging tree: {}",
                rel.display()
            )));
        }
        rel_parts.pop();
    }
    Ok(())
}

/// Build the signed payload for an extracted staging tree: the exact
/// manifest text plus a digest line per regular file in the tree, excluding
/// `signature.sig`. See [`payload_from_files`].
fn signed_payload(staging_dir: &Path, manifest_text: &str) -> CoreResult<Vec<u8>> {
    let mut files = Vec::new();
    collect_regular_files(staging_dir, &mut Vec::new(), &mut files)?;
    let mut in_memory = Vec::with_capacity(files.len());
    for (rel, abs) in files {
        let bytes = std::fs::read(&abs)?;
        in_memory.push((rel, bytes));
    }
    payload_from_files(manifest_text, &in_memory)
}

/// Return `true` only when `id` is a single normal path component:
/// non-empty and at most 64 chars, no `/`, `\`, `:` or NUL, and not `.` / `..`
/// (equivalently, `Path::components` yields exactly one `Component::Normal`).
/// The install pipeline joins this id into `plugins_dir/installed/<id>`, so a
/// traversal or absolute id must be rejected before any signature or
/// destination work happens. The uninstall command joins the same id into
/// `<vault>/.onq/plugins/installed/<id>` before `remove_dir_all`, so it must
/// reuse this rule. `:` is rejected lexically because drive prefixes like
/// `C:` are only path-special on Windows, and the rule must be platform-uniform
/// so an id accepted on one OS can never resolve outside `installed/` on another.
pub fn is_safe_plugin_id(id: &str) -> bool {
    if id.is_empty() || id.chars().count() > 64 {
        return false;
    }
    if id.contains('/') || id.contains('\\') || id.contains('\0') || id.contains(':') {
        return false;
    }
    let mut components = Path::new(id).components();
    matches!(
        (components.next(), components.next()),
        (Some(Component::Normal(_)), None)
    )
}

/// Install a `.tar.gz` plugin archive into `<plugins_dir>/installed/<id>/`.
///
/// See module docs for the on-disk layout. Returns the destination path on
/// success. Any error leaves no partial install behind. The temp
/// extraction is dropped before the function returns.
pub fn install(archive: &Path, plugins_dir: &Path) -> CoreResult<PathBuf> {
    install_with_limits(
        archive,
        plugins_dir,
        None,
        MAX_EXTRACTED_BYTES,
        MAX_EXTRACTED_ENTRIES,
    )
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
    install_with_limits(
        archive,
        plugins_dir,
        trust_anchor,
        MAX_EXTRACTED_BYTES,
        MAX_EXTRACTED_ENTRIES,
    )
}

/// Like [`install_with_trust_anchor`], but with explicit extraction caps so
/// the pre-verify disk bound is testable without a 1 GiB archive.
fn install_with_limits(
    archive: &Path,
    plugins_dir: &Path,
    trust_anchor: Option<&[u8; 32]>,
    max_bytes: u64,
    max_entries: usize,
) -> CoreResult<PathBuf> {
    // 1. Extract to a scratch tempdir (auto-cleaned on early return).
    let tmp = tempfile::tempdir()?;
    extract_tar_gz_with_limits(archive, tmp.path(), max_bytes, max_entries)?;

    // 2. Read + license-check the manifest.
    let manifest_path = tmp.path().join(MANIFEST_FILE);
    let manifest_text = std::fs::read_to_string(&manifest_path)?;
    // toml 1 parses `FromStr` as a single value. Manifests are documents.
    let manifest: toml::Value = toml::from_str(&manifest_text).map_err(CoreError::from)?;
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
    if !is_safe_plugin_id(id) {
        return Err(CoreError::Plugin(format!("invalid plugin id: {id:?}")));
    }
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
    //    The signed payload is the exact manifest text plus one digest
    //    line per regular file in the staging tree (excluding
    //    `signature.sig`), not the raw archive bytes, so the signer can
    //    produce a deterministic signature without a chicken-and-egg
    //    between the signed bytes and the signature file embedded inside
    //    the same archive. The plugin author signs that payload; the
    //    on-disk `signature.sig` carries that signature as-is.
    let sig = std::fs::read(tmp.path().join(SIGNATURE_FILE))?;
    let artifact = signed_payload(tmp.path(), &manifest_text)?;
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
/// `dest`) and non-regular entries (symlinks, hardlinks, FIFOs, devices)
/// are rejected before unpacking so a malicious archive cannot write
/// outside the staging directory before signature verification runs.
///
/// Extraction is capped at `MAX_EXTRACTED_BYTES` of decompressed tar stream
/// and `MAX_EXTRACTED_ENTRIES` entries, enforced before the manifest read
/// and signature verification so pre-verify disk use is bounded.
// Kept as the standard entry point; tests exercise the explicit-limit
// variant directly.
#[allow(dead_code)]
fn extract_tar_gz(archive: &Path, dest: &Path) -> CoreResult<()> {
    extract_tar_gz_with_limits(archive, dest, MAX_EXTRACTED_BYTES, MAX_EXTRACTED_ENTRIES)
}

/// Walk an io error's source chain looking for the size-limit marker
/// produced by [`BoundedReader`]. `tar` wraps the underlying error in an
/// opaque `failed to unpack` / `failed to read entire block` io error, so
/// the original message only survives in the chain.
fn is_size_limit_error(err: &std::io::Error) -> bool {
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(err);
    while let Some(e) = current {
        if e.to_string().contains(SIZE_LIMIT_MSG) {
            return true;
        }
        current = e.source();
    }
    false
}

/// Map an io error from the archive pipeline to a [`CoreError`], preserving
/// the plugin-flavored size-limit message when the byte cap trips.
fn map_extract_error(err: std::io::Error) -> CoreError {
    if is_size_limit_error(&err) {
        CoreError::Plugin(SIZE_LIMIT_MSG.to_string())
    } else {
        CoreError::Io(err)
    }
}

/// Like [`extract_tar_gz`], but with explicit caps so the limits are
/// testable without a 1 GiB archive.
fn extract_tar_gz_with_limits(
    archive: &Path,
    dest: &Path,
    max_bytes: u64,
    max_entries: usize,
) -> CoreResult<()> {
    std::fs::create_dir_all(dest)?;
    let file = File::open(archive)?;
    let decoder = BoundedReader {
        inner: GzDecoder::new(file),
        remaining: max_bytes,
    };
    let mut archive = Archive::new(decoder);
    let canonical_dest = dest.canonicalize().unwrap_or_else(|_| dest.to_path_buf());
    let mut entry_count = 0usize;
    let result = (|| -> CoreResult<()> {
        for entry in archive.entries()? {
            let mut entry = entry.map_err(map_extract_error)?;
            entry_count += 1;
            if entry_count > max_entries {
                return Err(CoreError::Plugin(
                    "plugin archive exceeds the entry count limit".into(),
                ));
            }
            let entry_path = entry.path().map_err(map_extract_error)?.into_owned();
            let safe_path = path_util::safe_join(&canonical_dest, &entry_path).map_err(|e| {
                // Preserve plugin-flavored errors for the install pipeline.
                CoreError::Plugin(e.to_string())
            })?;
            let entry_type = entry.header().entry_type();
            match entry_type {
                tar::EntryType::Regular | tar::EntryType::Directory => {}
                _ => {
                    return Err(CoreError::Plugin(format!(
                        "unsupported non-regular entry in plugin archive: {}",
                        entry_path.display()
                    )));
                }
            }
            if entry_type.is_dir() || entry_path.to_str().unwrap_or("").ends_with('/') {
                std::fs::create_dir_all(&safe_path)?;
            } else {
                if let Some(parent) = safe_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                entry.unpack(&safe_path).map_err(map_extract_error)?;
            }
        }
        Ok(())
    })();
    result?;
    // Drain the stream so the byte cap applies to the whole decompressed
    // stream, not just the entries that were unpacked. Tar writers pad the
    // tail with zero blocks that the entry iterator never reads.
    let mut decoder = archive.into_inner();
    let mut buf = [0u8; 8192];
    loop {
        match decoder.read(&mut buf) {
            Ok(0) => return Ok(()),
            Ok(_) => continue,
            Err(err) => return Err(map_extract_error(err)),
        }
    }
}

/// Recursively copy `src` into `dst`, creating `dst` if needed.
fn copy_dir_all(src: &Path, dst: &Path) -> CoreResult<()> {
    path_util::copy_path(src, dst)
}

/// Build a `.tar.gz` plugin archive in memory from manifest bytes, the
/// matching signature, and extra files. The returned bytes can be persisted
/// as the install package.
///
/// Layout: a single `manifest.toml`, a single `signature.sig`, plus an
/// optional list of arbitrary files (typically the plugin `.so`).
///
/// The `signature` argument must be an ed25519 signature over the signed
/// payload — the exact manifest text, a newline, then one
/// `<lowercase-hex sha256>  <relative path>\n` line per extra file (sorted
/// by relative path bytes, excluding `signature.sig`, which is not an extra
/// file). Signing only the manifest text produces an archive that fails
/// verification on install.
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
    use tempfile::TempDir;

    /// Build a self-signed plugin archive under `tmp` and return
    /// (archive_path, signing_key, manifest_text, pubkey). The fixture
    /// signs the signed payload — the exact manifest bytes plus one
    /// digest line per extra file — so `install_with_trust_anchor` can
    /// verify it deterministically without a chicken-and-egg between
    /// the embedded signature and the archive bytes that contain it.
    /// Returns the manifest text + pubkey so tests can pass the pubkey
    /// to `install_with_trust_anchor` and verify against the matching
    /// fixture key.
    fn build_signed_fixture(tmp: &Path, license: &str) -> (PathBuf, SigningKey, String, [u8; 32]) {
        let mut csprng = rand::rng();
        let key = SigningKey::generate(&mut csprng);
        let pubkey = key.verifying_key().to_bytes();
        let manifest = format!(
            "[plugin]\nid = \"acme.foo\"\nname = \"Foo\"\nversion = \"0.1.0\"\nlicense = {license:?}\ncapabilities = [\"read\"]\n",
        );
        let sig = sign_payload_for_extra_files(&manifest, &[("plugin.so", b"BINARY")], &key);
        let bytes =
            build_archive(&manifest, &sig, &[("plugin.so", b"BINARY")]).expect("build archive");
        let archive_path = tmp.join("plugin.tar.gz");
        std::fs::write(&archive_path, &bytes).expect("write archive");
        (archive_path, key, manifest, pubkey)
    }

    /// Sign the signed payload for `manifest` + `extra_files` with `key`.
    /// The manifest itself is a regular file in the extracted staging tree,
    /// so its digest line is part of the payload — the install pipeline
    /// recomputes the same payload from the extracted tree.
    fn sign_payload_for_extra_files(
        manifest: &str,
        extra_files: &[(&str, &[u8])],
        key: &SigningKey,
    ) -> Vec<u8> {
        let mut files: Vec<(PathBuf, Vec<u8>)> =
            vec![(PathBuf::from(MANIFEST_FILE), manifest.as_bytes().to_vec())];
        files.extend(
            extra_files
                .iter()
                .map(|(name, bytes)| (PathBuf::from(name), bytes.to_vec())),
        );
        let payload = payload_from_files(manifest, &files).expect("build payload");
        crate::signature::sign_for_test(&payload, key)
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
        let key = SigningKey::generate(&mut rand::rng());
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

    /// Build a self-signed archive whose manifest carries the given id and
    /// return (archive_path, pubkey). The signature is valid for the manifest
    /// bytes, so a rejection must come from the id check itself, not from
    /// signature verification.
    fn build_signed_archive_with_id(tmp: &Path, id: &str) -> (PathBuf, [u8; 32]) {
        let mut csprng = rand::rng();
        let key = SigningKey::generate(&mut csprng);
        let pubkey = key.verifying_key().to_bytes();
        let manifest = format!(
            "[plugin]\nid = {id:?}\nname = \"Foo\"\nversion = \"0.1.0\"\nlicense = \"GPL-3.0-only\"\ncapabilities = [\"read\"]\n",
        );
        let sig = sign_payload_for_extra_files(&manifest, &[("plugin.so", b"BINARY")], &key);
        let bytes =
            build_archive(&manifest, &sig, &[("plugin.so", b"BINARY")]).expect("build archive");
        let archive_path = tmp.join("plugin.tar.gz");
        std::fs::write(&archive_path, &bytes).expect("write archive");
        (archive_path, pubkey)
    }

    #[test]
    fn is_safe_plugin_id_accepts_simple_ids() {
        assert!(is_safe_plugin_id("pdf-tools"));
        assert!(is_safe_plugin_id("acme.foo"));
        assert!(is_safe_plugin_id("a..b"));
    }

    #[test]
    fn is_safe_plugin_id_rejects_unsafe_ids() {
        assert!(!is_safe_plugin_id("a/b"));
        assert!(!is_safe_plugin_id("a\\b"));
        assert!(!is_safe_plugin_id(".."));
        assert!(!is_safe_plugin_id("."));
        assert!(!is_safe_plugin_id(""));
        assert!(!is_safe_plugin_id("/tmp/onq-evil"));
        assert!(!is_safe_plugin_id("../escape"));
        assert!(!is_safe_plugin_id(&"x".repeat(65)));
        assert!(!is_safe_plugin_id("a\0b"));
        assert!(!is_safe_plugin_id("C:"));
        assert!(!is_safe_plugin_id("a:b"));
    }

    #[test]
    fn install_rejects_traversal_plugin_id() {
        let staging = TempDir::new().unwrap();
        let (archive, pubkey) = build_signed_archive_with_id(staging.path(), "../escape");
        let plugins_dir = TempDir::new().unwrap();
        let err =
            install_with_trust_anchor(&archive, plugins_dir.path(), Some(&pubkey)).unwrap_err();
        match err {
            CoreError::Plugin(msg) => {
                assert!(msg.contains("invalid plugin id"), "msg = {msg}");
            }
            other => panic!("expected Plugin error, got {other:?}"),
        }
        assert!(
            !plugins_dir.path().join("installed").exists(),
            "installed/ must not be created for an invalid id"
        );
        assert_eq!(
            std::fs::read_dir(plugins_dir.path()).unwrap().count(),
            0,
            "nothing may be created outside the plugins dir"
        );
    }

    #[test]
    fn install_rejects_absolute_plugin_id() {
        let staging = TempDir::new().unwrap();
        let (archive, pubkey) = build_signed_archive_with_id(staging.path(), "/tmp/onq-evil");
        let plugins_dir = TempDir::new().unwrap();
        let err =
            install_with_trust_anchor(&archive, plugins_dir.path(), Some(&pubkey)).unwrap_err();
        match err {
            CoreError::Plugin(msg) => {
                assert!(msg.contains("invalid plugin id"), "msg = {msg}");
            }
            other => panic!("expected Plugin error, got {other:?}"),
        }
        assert_eq!(
            std::fs::read_dir(plugins_dir.path()).unwrap().count(),
            0,
            "nothing may be created outside the plugins dir"
        );
    }

    #[test]
    fn install_rejects_empty_and_dot_ids() {
        for id in ["", ".", ".."] {
            let staging = TempDir::new().unwrap();
            let (archive, pubkey) = build_signed_archive_with_id(staging.path(), id);
            let plugins_dir = TempDir::new().unwrap();
            let err =
                install_with_trust_anchor(&archive, plugins_dir.path(), Some(&pubkey)).unwrap_err();
            match err {
                CoreError::Plugin(msg) => {
                    assert!(msg.contains("invalid plugin id"), "msg = {msg}");
                }
                other => panic!("expected Plugin error for id {id:?}, got {other:?}"),
            }
            assert_eq!(
                std::fs::read_dir(plugins_dir.path()).unwrap().count(),
                0,
                "nothing may be created for id {id:?}"
            );
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

    #[test]
    fn install_succeeds_for_untampered_archive() {
        let staging = TempDir::new().unwrap();
        let (archive, _key, _manifest, pubkey) =
            build_signed_fixture(staging.path(), "GPL-3.0-only");
        let plugins_dir = TempDir::new().unwrap();
        let dest = install_with_trust_anchor(&archive, plugins_dir.path(), Some(&pubkey))
            .expect("install");
        assert!(dest.join("manifest.toml").is_file());
        assert!(dest.join("plugin.so").is_file());
    }

    #[test]
    fn install_rejects_tampered_library() {
        let staging = TempDir::new().unwrap();
        let (archive, key, manifest, pubkey) = build_signed_fixture(staging.path(), "GPL-3.0-only");
        // Re-sign the original payload, then repack the archive with one
        // byte of the library flipped. The signature no longer covers the
        // archive contents, so verification must fail.
        let sig = sign_payload_for_extra_files(&manifest, &[("plugin.so", b"BINARY")], &key);
        let tampered = build_archive(&manifest, &sig, &[("plugin.so", b"BINARX")])
            .expect("build tampered archive");
        let tampered_path = staging.path().join("tampered.tar.gz");
        std::fs::write(&tampered_path, &tampered).unwrap();
        // The original archive still verifies (guards against a vacuous
        // rejection caused by a broken fixture).
        assert!(
            install_with_trust_anchor(&archive, &staging.path().join("guard"), Some(&pubkey))
                .is_ok(),
            "untampered fixture must install"
        );
        let plugins_dir = TempDir::new().unwrap();
        let err = install_with_trust_anchor(&tampered_path, plugins_dir.path(), Some(&pubkey))
            .unwrap_err();
        match err {
            CoreError::Plugin(_) => {}
            other => panic!("expected Plugin error, got {other:?}"),
        }
        assert!(
            !plugins_dir.path().join("installed").exists(),
            "no installed/<id> dir may be created for a tampered library"
        );
    }

    #[test]
    fn install_rejects_added_file() {
        let staging = TempDir::new().unwrap();
        let (archive, key, manifest, pubkey) = build_signed_fixture(staging.path(), "GPL-3.0-only");
        // Keep the same manifest + signature as the signed fixture, but
        // repack the archive with an extra unsigned file added after
        // signing. The payload now covers the added file, so the
        // signature no longer verifies.
        let sig = sign_payload_for_extra_files(&manifest, &[("plugin.so", b"BINARY")], &key);
        let tampered = build_archive(
            &manifest,
            &sig,
            &[("plugin.so", b"BINARY"), ("extra.txt", b"EXTRA")],
        )
        .expect("build archive with added file");
        let tampered_path = staging.path().join("added.tar.gz");
        std::fs::write(&tampered_path, &tampered).unwrap();
        // The untampered fixture must still install (guards against a
        // vacuous rejection).
        assert!(
            install_with_trust_anchor(&archive, &staging.path().join("guard"), Some(&pubkey))
                .is_ok(),
            "untampered fixture must install"
        );
        let plugins_dir = TempDir::new().unwrap();
        let err = install_with_trust_anchor(&tampered_path, plugins_dir.path(), Some(&pubkey))
            .unwrap_err();
        match err {
            CoreError::Plugin(_) => {}
            other => panic!("expected Plugin error, got {other:?}"),
        }
        assert!(
            !plugins_dir.path().join("installed").exists(),
            "no installed/<id> dir may be created for an added file"
        );
    }

    #[test]
    fn install_rejects_signature_sig_in_subdirectory() {
        let staging = TempDir::new().unwrap();
        let (_archive, key, manifest, pubkey) =
            build_signed_fixture(staging.path(), "GPL-3.0-only");
        let sig = sign_payload_for_extra_files(&manifest, &[("plugin.so", b"BINARY")], &key);
        let tampered = build_archive(
            &manifest,
            &sig,
            &[("plugin.so", b"BINARY"), ("evil/signature.sig", b"EVIL")],
        )
        .expect("build archive with nested signature.sig");
        let tampered_path = staging.path().join("nested.tar.gz");
        std::fs::write(&tampered_path, &tampered).unwrap();
        let plugins_dir = TempDir::new().unwrap();
        let err = install_with_trust_anchor(&tampered_path, plugins_dir.path(), Some(&pubkey))
            .unwrap_err();
        match err {
            CoreError::Plugin(msg) => assert!(msg.contains("signature.sig"), "msg = {msg}"),
            other => panic!("expected Plugin error, got {other:?}"),
        }
        assert!(
            !plugins_dir.path().join("installed").exists(),
            "no installed/<id> dir may be created for a nested signature.sig"
        );
    }

    #[test]
    fn signed_payload_is_order_independent() {
        let manifest = "[plugin]\nid = \"acme.foo\"\nname = \"Foo\"\nversion = \"0.1.0\"\nlicense = \"GPL-3.0-only\"\n";
        let files = [
            ("a.txt", b"aaa" as &[u8]),
            ("sub/b.txt", b"bbb" as &[u8]),
            ("c.txt", b"ccc" as &[u8]),
        ];
        let dir_a = TempDir::new().unwrap();
        let dir_b = TempDir::new().unwrap();
        // Create dir_a in order 0, 1, 2; dir_b in reverse order.
        for (rel, bytes) in &files {
            let path = dir_a.path().join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, bytes).unwrap();
        }
        for (rel, bytes) in files.iter().rev() {
            let path = dir_b.path().join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, bytes).unwrap();
        }
        let payload_a = signed_payload(dir_a.path(), manifest).expect("payload A");
        let payload_b = signed_payload(dir_b.path(), manifest).expect("payload B");
        assert_eq!(
            payload_a, payload_b,
            "payload must not depend on file creation order"
        );
        // Payload shape: exact manifest bytes, a newline, then one
        // `<64-hex>  <rel-path>\n` line per file, sorted by path bytes.
        assert!(payload_a.starts_with(manifest.as_bytes()));
        let rest = &payload_a[manifest.len() + 1..];
        let lines: Vec<&str> = std::str::from_utf8(rest).unwrap().lines().collect();
        assert_eq!(lines.len(), files.len(), "one digest line per file");
        let mut sorted = files.to_vec();
        sorted.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        for (line, (rel, bytes)) in lines.iter().zip(sorted.iter()) {
            let expected_hex = sha256_hex(bytes);
            assert_eq!(line.len(), 64 + 2 + rel.len());
            assert!(line.starts_with(&expected_hex), "line = {line}");
            assert!(line.ends_with(rel), "line = {line}");
        }
    }

    /// Build an archive containing a planted symlink entry (`evil.so ->
    /// plugin.so`) alongside the manifest, signature, and library. The
    /// signature is valid for the payload the installer would compute if
    /// the symlink were ignored, but the installer must reject the symlink
    /// before it ever reaches verification.
    fn build_archive_with_symlink(manifest: &str, sig: &[u8]) -> Vec<u8> {
        let buf = Vec::new();
        let encoder = flate2::write::GzEncoder::new(buf, flate2::Compression::default());
        let mut tar = tar::Builder::new(encoder);
        append_bytes(&mut tar, MANIFEST_FILE, manifest.as_bytes()).unwrap();
        append_bytes(&mut tar, SIGNATURE_FILE, sig).unwrap();
        append_bytes(&mut tar, "plugin.so", b"BINARY").unwrap();
        let mut header = tar::Header::new_gnu();
        header.set_path("evil.so").unwrap();
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_link_name("plugin.so").unwrap();
        header.set_size(0);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append(&header, &mut std::io::empty()).unwrap();
        tar.finish().unwrap();
        let encoder = tar.into_inner().unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn install_rejects_symlink_entry() {
        let staging = TempDir::new().unwrap();
        let key = SigningKey::generate(&mut rand::rng());
        let pubkey = key.verifying_key().to_bytes();
        let manifest = "[plugin]\nid = \"acme.foo\"\nname = \"Foo\"\nversion = \"0.1.0\"\nlicense = \"GPL-3.0-only\"\ncapabilities = [\"read\"]\n";
        let sig = sign_payload_for_extra_files(manifest, &[("plugin.so", b"BINARY")], &key);
        let bytes = build_archive_with_symlink(manifest, &sig);
        let archive_path = staging.path().join("symlink.tar.gz");
        std::fs::write(&archive_path, &bytes).unwrap();

        let plugins_dir = TempDir::new().unwrap();
        let err = install_with_trust_anchor(&archive_path, plugins_dir.path(), Some(&pubkey))
            .unwrap_err();
        match err {
            CoreError::Plugin(msg) => {
                assert!(msg.contains("non-regular"), "msg = {msg}");
                assert!(msg.contains("evil.so"), "msg = {msg}");
            }
            other => panic!("expected Plugin error, got {other:?}"),
        }
        assert!(
            !plugins_dir.path().join("installed").exists(),
            "no installed/<id> dir may be created for a symlink plant"
        );
    }

    #[test]
    fn install_rejects_symlink_write_escape_before_verify() {
        let staging = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        let key = SigningKey::generate(&mut rand::rng());
        let pubkey = key.verifying_key().to_bytes();
        let manifest = "[plugin]\nid = \"acme.foo\"\nname = \"Foo\"\nversion = \"0.1.0\"\nlicense = \"GPL-3.0-only\"\ncapabilities = [\"read\"]\n";
        let sig = sign_payload_for_extra_files(manifest, &[("plugin.so", b"BINARY")], &key);

        let buf = Vec::new();
        let encoder = flate2::write::GzEncoder::new(buf, flate2::Compression::default());
        let mut tar = tar::Builder::new(encoder);
        let mut link_hdr = tar::Header::new_gnu();
        link_hdr.set_path("escape").unwrap();
        link_hdr.set_entry_type(tar::EntryType::Symlink);
        link_hdr.set_link_name(outside.path()).unwrap();
        link_hdr.set_size(0);
        link_hdr.set_mode(0o755);
        link_hdr.set_cksum();
        tar.append(&link_hdr, &mut std::io::empty()).unwrap();
        append_bytes(&mut tar, "escape/pwned.txt", b"PWNED").unwrap();
        append_bytes(&mut tar, MANIFEST_FILE, manifest.as_bytes()).unwrap();
        append_bytes(&mut tar, SIGNATURE_FILE, &sig).unwrap();
        tar.finish().unwrap();
        let bytes = tar.into_inner().unwrap().finish().unwrap();

        let archive_path = staging.path().join("escape.tar.gz");
        std::fs::write(&archive_path, &bytes).unwrap();
        let plugins_dir = TempDir::new().unwrap();
        let err = install_with_trust_anchor(&archive_path, plugins_dir.path(), Some(&pubkey))
            .unwrap_err();
        assert!(matches!(err, CoreError::Plugin(_)));
        assert!(
            !outside.path().join("pwned.txt").exists(),
            "symlink write escape must be blocked before unpacking"
        );
    }

    /// Build a gzipped tar of `n` regular entries `f{i}.txt`, each an
    /// `entry_size`-byte zero-filled file, written to a temp file. Returns
    /// the archive path.
    fn build_zero_filled_archive(tmp: &Path, n: usize, entry_size: usize) -> PathBuf {
        let buf = Vec::new();
        let encoder = flate2::write::GzEncoder::new(buf, flate2::Compression::default());
        let mut tar = tar::Builder::new(encoder);
        let zeros = vec![0u8; entry_size];
        for i in 0..n {
            let mut header = tar::Header::new_gnu();
            header.set_size(entry_size as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, format!("f{i}.txt"), &zeros[..])
                .unwrap();
        }
        tar.finish().unwrap();
        let encoder = tar.into_inner().unwrap();
        let bytes = encoder.finish().unwrap();
        let path = tmp.join(format!("zero-{n}-{entry_size}.tar.gz"));
        std::fs::write(&path, &bytes).unwrap();
        path
    }

    /// Recursively sum the sizes of all files under `dir`.
    fn sum_file_sizes(dir: &Path) -> u64 {
        let mut total = 0u64;
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_type = entry.file_type().unwrap();
            if file_type.is_dir() {
                total += sum_file_sizes(&path);
            } else if file_type.is_file() {
                total += std::fs::metadata(&path).unwrap().len();
            }
        }
        total
    }

    /// Measure the decompressed byte length of a `.tar.gz` archive by
    /// streaming it through a `GzDecoder` into a counter.
    fn decompressed_length(archive: &Path) -> u64 {
        let file = File::open(archive).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut total = 0u64;
        let mut buf = [0u8; 8192];
        loop {
            let n = decoder.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            total += n as u64;
        }
        total
    }

    #[test]
    fn extract_rejects_stream_over_decompressed_byte_limit() {
        let staging = TempDir::new().unwrap();
        let archive = build_zero_filled_archive(staging.path(), 4, 64 * 1024);
        let dest = staging.path().join("dest");
        let err = extract_tar_gz_with_limits(&archive, &dest, 128 * 1024, 10_000).unwrap_err();
        assert!(
            err.to_string().contains("decompressed size limit"),
            "msg = {err}"
        );
        let total = sum_file_sizes(&dest);
        assert!(
            total <= 128 * 1024,
            "extracted bytes {total} exceed the byte cap"
        );
    }

    #[test]
    fn extract_byte_limit_boundary_is_inclusive() {
        let staging = TempDir::new().unwrap();
        let archive = build_zero_filled_archive(staging.path(), 3, 100);
        let len = decompressed_length(&archive);

        let dest = staging.path().join("dest-ok");
        extract_tar_gz_with_limits(&archive, &dest, len, 10_000)
            .expect("exact-size limit succeeds");
        for i in 0..3 {
            assert!(dest.join(format!("f{i}.txt")).is_file());
        }

        let dest = staging.path().join("dest-fail");
        let err = extract_tar_gz_with_limits(&archive, &dest, len - 1, 10_000).unwrap_err();
        assert!(
            err.to_string().contains("decompressed size limit"),
            "msg = {err}"
        );
    }

    #[test]
    fn extract_rejects_archive_over_entry_count_limit() {
        let staging = TempDir::new().unwrap();
        let archive = build_zero_filled_archive(staging.path(), 12, 1);

        let dest = staging.path().join("dest-limited");
        let err = extract_tar_gz_with_limits(&archive, &dest, 1 << 30, 10).unwrap_err();
        assert!(err.to_string().contains("entry count limit"), "msg = {err}");
        assert!(!dest.join("f10.txt").exists(), "f10.txt must not exist");
        assert!(!dest.join("f11.txt").exists(), "f11.txt must not exist");

        let dest = staging.path().join("dest-ok");
        extract_tar_gz_with_limits(&archive, &dest, 1 << 30, 12).expect("12 entries succeed");
        for i in 0..12 {
            assert!(dest.join(format!("f{i}.txt")).is_file(), "f{i}.txt missing");
        }
    }

    #[test]
    fn install_rejects_oversized_archive_before_signature_check() {
        let staging = TempDir::new().unwrap();
        // Unsigned archive: no manifest.toml, no signature.sig.
        let archive = build_zero_filled_archive(staging.path(), 3, 64 * 1024);
        let plugins_dir = TempDir::new().unwrap();
        let err = install_with_limits(&archive, plugins_dir.path(), None, 128 * 1024, 10_000)
            .unwrap_err();
        assert!(
            err.to_string().contains("decompressed size limit"),
            "msg = {err}"
        );
    }
}
