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
        if !matches_filter(&prompt.fm.tags, prompt.fm.folder.as_deref(), prompt.fm.favorite, filter)
        {
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
        if folder != Some(want) {
            return false;
        }
    }
    if !filter.tags_any.is_empty() {
        let has = filter.tags_any.iter().any(|t| tags.iter().any(|pt| pt == t));
        if !has {
            return false;
        }
    }
    true
}

/// Export a single prompt id list (used by multi-select UI).
pub fn export_prompt_ids(
    vault: &Vault,
    dest: &Path,
    ids: &[PromptId],
) -> CoreResult<ExportReport> {
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
        let id = PromptId::new();
        let now = Utc::now();
        vault
            .write(&Prompt {
                fm: Frontmatter {
                    id: id.clone(),
                    title: title.into(),
                    folder: Some("work".into()),
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
}
