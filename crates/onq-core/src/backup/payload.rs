//! Vault tree ↔ gzipped-tar payload (the bytes inside a `.onqbak` container).
//!
//! Path-traversal entries are rejected via [`crate::path_util::safe_join`].
//! Packing skips import-recovery leftovers ([`super::layout::should_skip_pack_entry`]).

use std::fs::File;
use std::io::Write;
use std::path::Path;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Archive, Builder, Header};

use super::layout;
use crate::error::{CoreError, CoreResult};
use crate::path_util;

/// Walk `vault_path` and produce a gzipped tar suitable for the container.
pub fn pack_tree(vault_path: &Path) -> CoreResult<Vec<u8>> {
    let mut raw = Vec::new();
    {
        let enc = GzEncoder::new(&mut raw, Compression::default());
        let mut builder = Builder::new(enc);
        append_dir(&mut builder, vault_path, Path::new(""))?;
        let enc = builder
            .into_inner()
            .map_err(|e| CoreError::Other(format!("tar finish: {e}")))?;
        enc.finish()
            .map_err(|e| CoreError::Other(format!("gzip finish: {e}")))?;
    }
    Ok(raw)
}

/// Unpack a gzipped-tar payload into `dest`, creating it if needed.
pub fn unpack_tree(payload: &[u8], dest: &Path) -> CoreResult<()> {
    std::fs::create_dir_all(dest)?;
    let decoder = GzDecoder::new(payload);
    let mut archive = Archive::new(decoder);
    let canonical_dest = dest.canonicalize().unwrap_or_else(|_| dest.to_path_buf());
    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?.into_owned();
        let safe_path = path_util::safe_join(&canonical_dest, &entry_path)?;
        if entry.header().entry_type().is_dir()
            || entry_path.to_str().unwrap_or("").ends_with('/')
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

fn append_dir<W: Write>(
    builder: &mut Builder<W>,
    abs: &Path,
    rel: &Path,
) -> CoreResult<()> {
    for entry in std::fs::read_dir(abs)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if layout::should_skip_pack_entry(&name_str) {
            continue;
        }
        let child_abs = entry.path();
        let child_rel = rel.join(&name);
        let meta = entry.metadata()?;
        let rel_str = child_rel.to_string_lossy().replace('\\', "/");
        if meta.is_dir() {
            let mut header = Header::new_gnu();
            header.set_entry_type(tar::EntryType::Directory);
            header.set_mode(0o755);
            header.set_size(0);
            header.set_cksum();
            builder
                .append_data(&mut header, format!("{rel_str}/"), std::io::empty())
                .map_err(|e| CoreError::Other(format!("tar append dir: {e}")))?;
            append_dir(builder, &child_abs, &child_rel)?;
        } else if meta.is_file() {
            let mut file = File::open(&child_abs)?;
            builder
                .append_file(&rel_str, &mut file)
                .map_err(|e| CoreError::Other(format!("tar append file: {e}")))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::layout::{database_path, is_vault_root, salt_path};
    use crate::path_util;
    use tempfile::TempDir;

    fn seed(root: &Path) {
        std::fs::create_dir_all(database_path(root)).unwrap();
        std::fs::write(salt_path(root), [2u8; 32]).unwrap();
        std::fs::write(root.join("note.md"), b"hi").unwrap();
    }

    #[test]
    fn pack_unpack_preserves_files() {
        let src = TempDir::new().unwrap();
        seed(src.path());
        let payload = pack_tree(src.path()).unwrap();
        let dest = TempDir::new().unwrap();
        unpack_tree(&payload, dest.path()).unwrap();
        assert!(is_vault_root(dest.path()));
        assert_eq!(std::fs::read(dest.path().join("note.md")).unwrap(), b"hi");
    }

    #[test]
    fn unpack_uses_path_util_guard() {
        // tar's Builder refuses `..` / absolute paths, so the escape guard is
        // covered directly via path_util (same call site as unpack_tree).
        let base = Path::new("/tmp/onq-stage");
        assert!(path_util::safe_join(base, Path::new("../x")).is_err());
        assert!(path_util::safe_join(base, Path::new("ok/a.md")).is_ok());
    }

    #[test]
    fn pack_skips_pre_import_dirs() {
        let src = TempDir::new().unwrap();
        seed(src.path());
        let junk = src.path().join(".onq-pre-import-1");
        std::fs::create_dir_all(&junk).unwrap();
        std::fs::write(junk.join("x"), b"no").unwrap();
        let payload = pack_tree(src.path()).unwrap();
        let dest = TempDir::new().unwrap();
        unpack_tree(&payload, dest.path()).unwrap();
        assert!(!dest.path().join(".onq-pre-import-1").exists());
    }
}
