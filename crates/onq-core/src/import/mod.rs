//! Bulk prompt import from external formats into a vault.

mod chatgpt_export;
mod json_array;
mod markdown_dir;

pub use chatgpt_export::import_chatgpt_export;
pub use json_array::import_json_array;
pub use markdown_dir::import_markdown_dir;

use crate::error::CoreResult;
use crate::vault::Vault;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub created: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportFormat {
    Auto,
    MarkdownDir,
    JsonArray,
    ChatGpt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnConflict {
    Skip,
    Replace,
}

/// Import prompts into `vault` from `path` using `format`.
pub fn import_prompts(
    vault: &Vault,
    path: &std::path::Path,
    format: ImportFormat,
    on_conflict: OnConflict,
) -> CoreResult<ImportReport> {
    let resolved = match format {
        ImportFormat::Auto => detect_format(path),
        other => other,
    };
    match resolved {
        ImportFormat::MarkdownDir => import_markdown_dir(vault, path, on_conflict),
        ImportFormat::JsonArray => import_json_array(vault, path, on_conflict),
        ImportFormat::ChatGpt => import_chatgpt_export(vault, path, on_conflict),
        ImportFormat::Auto => unreachable!("detect_format never returns Auto"),
    }
}

fn detect_format(path: &std::path::Path) -> ImportFormat {
    if path.is_dir() {
        return ImportFormat::MarkdownDir;
    }
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if name.contains("conversations") || name.ends_with(".zip") {
        return ImportFormat::ChatGpt;
    }
    if name.ends_with(".json") {
        return ImportFormat::JsonArray;
    }
    ImportFormat::MarkdownDir
}
