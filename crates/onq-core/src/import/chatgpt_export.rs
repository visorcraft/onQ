//! Best-effort ChatGPT export importer.
//!
//! Accepts either:
//! - a `conversations.json` file (OpenAI data export shape, loosely)
//! - a directory containing `conversations.json`

use std::path::Path;

use chrono::Utc;
use serde::Deserialize;

use crate::error::{CoreError, CoreResult};
use crate::frontmatter::Frontmatter;
use crate::import::{ImportReport, OnConflict};
use crate::ulid::PromptId;
use crate::vault::{Prompt, Vault};

#[derive(Debug, Deserialize)]
struct Conversation {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    mapping: Option<serde_json::Value>,
    /// Present in ChatGPT export JSON; not required for import.
    #[serde(default)]
    #[serde(rename = "current_node")]
    _current_node: Option<String>,
}

/// Import ChatGPT conversations as one prompt per conversation title+last message.
pub fn import_chatgpt_export(
    vault: &Vault,
    path: &Path,
    on_conflict: OnConflict,
) -> CoreResult<ImportReport> {
    let json_path = if path.is_dir() {
        path.join("conversations.json")
    } else {
        path.to_path_buf()
    };
    if !json_path.is_file() {
        let mut report = ImportReport::default();
        report.errors.push(format!(
            "conversations.json not found at {}",
            json_path.display()
        ));
        return Ok(report);
    }
    let raw = std::fs::read_to_string(&json_path)?;
    let conversations: Vec<Conversation> =
        serde_json::from_str(&raw).map_err(|e| CoreError::Other(format!("chatgpt import: {e}")))?;

    let mut report = ImportReport::default();
    for conv in conversations {
        let body = extract_body(&conv).unwrap_or_default();
        let title = conv
            .title
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| "ChatGPT conversation".into());
        let body = if body.is_empty() { title.clone() } else { body };
        let now = Utc::now();
        let id = PromptId::new();
        if vault.read(&id).is_ok() && matches!(on_conflict, OnConflict::Skip) {
            report.skipped += 1;
            continue;
        }
        let fm = Frontmatter {
            id,
            title,
            folder: Some("imported/chatgpt".into()),
            tags: vec!["imported".into(), "chatgpt".into()],
            favorite: false,
            locked: false,
            created: now,
            updated: now,
        };
        match vault.write(&Prompt { fm, body }) {
            Ok(()) => report.created += 1,
            Err(e) => report.errors.push(e.to_string()),
        }
    }
    Ok(report)
}

fn extract_body(conv: &Conversation) -> Option<String> {
    let mapping = conv.mapping.as_ref()?.as_object()?;
    // Prefer walking current_node chain; fall back to collecting message parts.
    let mut texts = Vec::new();
    for node in mapping.values() {
        let message = node.get("message")?;
        let role = message
            .get("author")
            .and_then(|a| a.get("role"))
            .and_then(|r| r.as_str())
            .unwrap_or("");
        if role != "user" && role != "assistant" {
            continue;
        }
        if let Some(parts) = message
            .get("content")
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.as_array())
        {
            for part in parts {
                if let Some(s) = part.as_str() {
                    if !s.trim().is_empty() {
                        texts.push(format!("**{role}:** {s}"));
                    }
                }
            }
        }
    }
    if texts.is_empty() {
        None
    } else {
        Some(texts.join("\n\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn imports_minimal_conversations_json() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        let json = dir.path().join("conversations.json");
        std::fs::write(
            &json,
            r#"[{
              "title": "Test chat",
              "mapping": {
                "n1": {
                  "message": {
                    "author": {"role": "user"},
                    "content": {"parts": ["Hello model"]}
                  }
                }
              }
            }]"#,
        )
        .unwrap();
        let report = import_chatgpt_export(&vault, &json, OnConflict::Skip).unwrap();
        assert_eq!(report.created, 1);
        assert!(report.errors.is_empty());
    }
}
