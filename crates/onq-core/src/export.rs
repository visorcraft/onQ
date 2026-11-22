//! Curated prompt export to a destination directory of Markdown files.

use std::path::{Path, PathBuf};

use crate::error::CoreResult;
use crate::frontmatter;
use crate::ulid::PromptId;
use crate::vault::Vault;

#[derive(Debug, Clone, Default)]
pub struct ExportFilter {
    /// If non-empty, prompt must include at least one of these tags.
    pub tags_any: Vec<String>,
    /// If set, prompt folder must equal this value (exact).
    pub folder: Option<String>,
    /// If true, only favorites.
    pub favorites_only: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExportReport {
    pub exported: usize,
    pub skipped: usize,
    pub paths: Vec<PathBuf>,
}

/// Write matching prompts as `.md` files into `dest` (created if needed).
pub fn export_prompts(
    vault: &Vault,
    dest: &Path,
    filter: &ExportFilter,
) -> CoreResult<ExportReport> {
    std::fs::create_dir_all(dest)?;
    let mut report = ExportReport::default();
    for id in vault.list()? {
        let prompt = match vault.read(&id) {
            Ok(p) => p,
            Err(_) => {
                report.skipped += 1;
                continue;
            }
        };
        if !matches_filter(
            &prompt.fm.tags,
            prompt.fm.folder.as_deref(),
            prompt.fm.favorite,
            filter,
        ) {
            report.skipped += 1;
            continue;
        }
        let file = dest.join(format!("{}.md", id.as_str()));
        let rendered = frontmatter::render(&prompt.fm, &prompt.body);
        crate::vault::atomic_write(&file, rendered.as_bytes())?;
        report.exported += 1;
        report.paths.push(file);
    }
    Ok(report)
}

/// Match prompt against the filter. Tag and folder comparisons are trimmed and
/// case-sensitive, mirroring the library tag-chip filter.
fn matches_filter(
    tags: &[String],
    folder: Option<&str>,
    favorite: bool,
    filter: &ExportFilter,
) -> bool {
    if filter.favorites_only && !favorite {
        return false;
    }
    if let Some(want) = filter.folder.as_deref() {
        if folder.map(str::trim) != Some(want.trim()) {
            return false;
        }
    }
    if !filter.tags_any.is_empty() {
        let has = filter
            .tags_any
            .iter()
            .any(|t| tags.iter().any(|pt| pt.trim() == t.trim()));
        if !has {
            return false;
        }
    }
    true
}

/// Export a single prompt id list (used by multi-select UI).
pub fn export_prompt_ids(vault: &Vault, dest: &Path, ids: &[PromptId]) -> CoreResult<ExportReport> {
    std::fs::create_dir_all(dest)?;
    let mut report = ExportReport::default();
    for id in ids {
        let prompt = match vault.read(id) {
            Ok(p) => p,
            Err(_) => {
                report.skipped += 1;
                continue;
            }
        };
        let file = dest.join(format!("{}.md", id.as_str()));
        let rendered = frontmatter::render(&prompt.fm, &prompt.body);
        crate::vault::atomic_write(&file, rendered.as_bytes())?;
        report.exported += 1;
        report.paths.push(file);
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontmatter::Frontmatter;
    use crate::vault::Prompt;
    use chrono::Utc;
    use tempfile::TempDir;

    fn seed(vault: &Vault, title: &str, tags: &[&str], favorite: bool) -> PromptId {
        seed_with(vault, title, tags, favorite, Some("work"))
    }

    fn seed_with(
        vault: &Vault,
        title: &str,
        tags: &[&str],
        favorite: bool,
        folder: Option<&str>,
    ) -> PromptId {
        let id = PromptId::new();
        let now = Utc::now();
        vault
            .write(&Prompt {
                fm: Frontmatter {
                    id: id.clone(),
                    title: title.into(),
                    folder: folder.map(|f| f.to_string()),
                    tags: tags.iter().map(|s| (*s).to_string()).collect(),
                    favorite,
                    locked: false,
                    created: now,
                    updated: now,
                },
                body: format!("body of {title}"),
            })
            .unwrap();
        id
    }

    #[test]
    fn export_filters_by_tag() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        seed(&vault, "A", &["writing"], false);
        seed(&vault, "B", &["code"], true);
        let dest = dir.path().join("out");
        let report = export_prompts(
            &vault,
            &dest,
            &ExportFilter {
                tags_any: vec!["writing".into()],
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(report.exported, 1);
        assert_eq!(report.skipped, 1);
    }

    #[test]
    fn export_matches_tag_with_surrounding_whitespace() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        seed(&vault, "A", &[" writing "], false);
        let dest = dir.path().join("out");
        let report = export_prompts(
            &vault,
            &dest,
            &ExportFilter {
                tags_any: vec!["writing".into()],
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(report.exported, 1);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn export_filter_tag_with_whitespace_matches_stored_tag() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        seed(&vault, "A", &["code"], false);
        let dest = dir.path().join("out");
        let report = export_prompts(
            &vault,
            &dest,
            &ExportFilter {
                tags_any: vec!["  code ".into()],
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(report.exported, 1);
    }

    #[test]
    fn export_tag_match_stays_case_sensitive() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        seed(&vault, "A", &["writing"], false);
        let dest = dir.path().join("out");
        let report = export_prompts(
            &vault,
            &dest,
            &ExportFilter {
                tags_any: vec!["Writing".into()],
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(report.exported, 0);
        assert_eq!(report.skipped, 1);
    }

    #[test]
    fn export_folder_filter_trims_both_sides() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        seed_with(&vault, "A", &["x"], false, Some(" work "));
        seed_with(&vault, "B", &["x"], false, Some("work"));
        seed_with(&vault, "C", &["x"], false, None);

        let dest = dir.path().join("out1");
        let report = export_prompts(
            &vault,
            &dest,
            &ExportFilter {
                folder: Some("work".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(report.exported, 2);
        assert_eq!(report.skipped, 1);

        let dest2 = dir.path().join("out2");
        let report2 = export_prompts(
            &vault,
            &dest2,
            &ExportFilter {
                folder: Some(" work ".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(report2.exported, 2);
        assert_eq!(report2.skipped, 1);
    }
}
