use std::path::Path;

use chrono::Utc;

use crate::error::CoreResult;
use crate::folder_path;
use crate::frontmatter::{self, Frontmatter};
use crate::import::{ImportReport, OnConflict};
use crate::ulid::PromptId;
use crate::vault::{Prompt, Vault};

/// Import every `*.md` under `dir` (recursive, depth-capped).
pub fn import_markdown_dir(
    vault: &Vault,
    dir: &Path,
    on_conflict: OnConflict,
) -> CoreResult<ImportReport> {
    let mut report = ImportReport::default();
    if !dir.is_dir() {
        report
            .errors
            .push(format!("not a directory: {}", dir.display()));
        return Ok(report);
    }
    walk(dir, vault, on_conflict, &mut report, 0)?;
    Ok(report)
}

fn walk(
    dir: &Path,
    vault: &Vault,
    on_conflict: OnConflict,
    report: &mut ImportReport,
    depth: usize,
) -> CoreResult<()> {
    if depth > 4 {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, vault, on_conflict, report, depth + 1)?;
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        match import_one(vault, &path, on_conflict) {
            Ok(true) => report.created += 1,
            Ok(false) => report.skipped += 1,
            Err(e) => report.errors.push(format!("{}: {e}", path.display())),
        }
    }
    Ok(())
}

/// Returns Ok(true) when a prompt was written, Ok(false) when skipped.
fn import_one(vault: &Vault, path: &Path, on_conflict: OnConflict) -> CoreResult<bool> {
    let raw = std::fs::read_to_string(path)?;
    let (mut fm, body) = match frontmatter::parse(&raw) {
        Ok(pair) => pair,
        Err(_) => {
            let title = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Imported".into());
            let now = Utc::now();
            (
                Frontmatter {
                    id: PromptId::new(),
                    title,
                    folder: None,
                    tags: vec![],
                    favorite: false,
                    locked: false,
                    created: now,
                    updated: now,
                },
                raw,
            )
        }
    };
    if fm.id.as_str().is_empty() {
        fm.id = PromptId::new();
    }
    if let Some(raw) = fm.folder.take() {
        if !raw.trim().is_empty() {
            fm.folder = Some(folder_path::normalize(&raw)?);
        }
    }
    let exists = vault.read(&fm.id).is_ok();
    if exists && matches!(on_conflict, OnConflict::Skip) {
        return Ok(false);
    }
    fm.updated = Utc::now();
    vault.write(&Prompt { fm, body })?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn imports_plain_markdown_without_frontmatter() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("hello.md"), "# Hello\n\nbody").unwrap();
        let report = import_markdown_dir(&vault, &src, OnConflict::Skip).unwrap();
        assert_eq!(report.created, 1);
        assert!(report.errors.is_empty());
        assert_eq!(vault.list().unwrap().len(), 1);
    }

    fn frontmatter_doc(id: &str, folder: &str) -> String {
        format!(
            "---\nid: {id}\ntitle: T\nfolder: \"{folder}\"\ncreated: 2026-07-18T10:30:00Z\nupdated: 2026-07-18T10:30:00Z\n---\n\nbody"
        )
    }

    #[test]
    fn normalizes_frontmatter_folder_on_import() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        let id = PromptId::new();
        std::fs::write(src.join("note.md"), frontmatter_doc(id.as_str(), "A // B")).unwrap();
        let report = import_markdown_dir(&vault, &src, OnConflict::Skip).unwrap();
        assert_eq!(report.created, 1);
        assert!(report.errors.is_empty());
        let prompt = vault.read(&id).unwrap();
        assert_eq!(prompt.fm.folder.as_deref(), Some("A/B"));
    }

    #[test]
    fn imports_traversal_id_file_as_plain_markdown() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("evil.md"),
            "---\nid: ../../evil\ntitle: T\ncreated: 2026-07-18T10:30:00Z\nupdated: 2026-07-18T10:30:00Z\n---\n\nbody",
        )
        .unwrap();
        let report = import_markdown_dir(&vault, &src, OnConflict::Skip).unwrap();
        assert_eq!(report.created, 1);
        assert!(report.errors.is_empty());
        let ids = vault.list().unwrap();
        assert_eq!(ids.len(), 1);
        assert!(PromptId::is_valid_str(ids[0].as_str()));
        assert!(!dir.path().join("evil.md").exists());
    }

    #[test]
    fn reports_file_with_invalid_folder() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        let id = PromptId::new();
        std::fs::write(src.join("bad.md"), frontmatter_doc(id.as_str(), "..")).unwrap();
        let report = import_markdown_dir(&vault, &src, OnConflict::Skip).unwrap();
        assert_eq!(report.created, 0);
        assert_eq!(report.errors.len(), 1);
        assert!(report.errors[0].contains("bad.md"));
        assert!(vault.read(&id).is_err());
    }
}
