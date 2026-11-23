use std::path::Path;

use chrono::Utc;
use serde::Deserialize;

use crate::error::{CoreError, CoreResult};
use crate::folder_path;
use crate::frontmatter::Frontmatter;
use crate::import::{read_json_source, ImportReport, OnConflict};
use crate::ulid::PromptId;
use crate::vault::{Prompt, Vault};

#[derive(Debug, Deserialize)]
struct JsonPrompt {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    folder: Option<String>,
}

/// Import a JSON file containing an array of prompt objects.
pub fn import_json_array(
    vault: &Vault,
    path: &Path,
    on_conflict: OnConflict,
) -> CoreResult<ImportReport> {
    let mut report = ImportReport::default();
    let raw = read_json_source(path, "json import")?;
    let items: Vec<JsonPrompt> =
        serde_json::from_str(&raw).map_err(|e| CoreError::Other(format!("json import: {e}")))?;
    for item in items {
        match write_item(vault, item, on_conflict) {
            Ok(true) => report.created += 1,
            Ok(false) => report.skipped += 1,
            Err(e) => report.errors.push(e.to_string()),
        }
    }
    Ok(report)
}

fn write_item(vault: &Vault, item: JsonPrompt, on_conflict: OnConflict) -> CoreResult<bool> {
    let now = Utc::now();
    let id = match item.id.as_deref() {
        Some(s) if !s.is_empty() => {
            PromptId::from_string(s.to_string()).map_err(|e| CoreError::Other(e.to_string()))?
        }
        _ => PromptId::new(),
    };
    let exists = vault.read(&id).is_ok();
    if exists && matches!(on_conflict, OnConflict::Skip) {
        return Ok(false);
    }
    let title = item
        .title
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "Imported".into());
    let folder = item.folder.filter(|f| !f.trim().is_empty());
    let folder = match folder {
        Some(raw) => Some(folder_path::normalize(&raw)?),
        None => None,
    };
    let body = item.body.unwrap_or_default();
    let fm = Frontmatter {
        id,
        title,
        folder,
        tags: item.tags.unwrap_or_default(),
        favorite: false,
        locked: false,
        created: now,
        updated: now,
    };
    vault.write(&Prompt { fm, body })?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn imports_json_array_with_utf8_bom() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        let json_path = dir.path().join("prompts.json");
        let mut bytes = b"\xEF\xBB\xBF".to_vec();
        bytes.extend_from_slice(br#"[{"title":"A","body":"one"}]"#);
        std::fs::write(&json_path, bytes).unwrap();
        let report = import_json_array(&vault, &json_path, OnConflict::Skip).unwrap();
        assert_eq!(report.created, 1);
        assert!(report.errors.is_empty());
        let ids = vault.list().unwrap();
        assert_eq!(ids.len(), 1);
        let prompt = vault.read(&ids[0]).unwrap();
        assert_eq!(prompt.fm.title, "A");
    }

    #[test]
    fn invalid_utf8_json_is_clear_error() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        let json_path = dir.path().join("prompts.json");
        std::fs::write(&json_path, b"[{\"title\":\"\xff\"}]").unwrap();
        let err = import_json_array(&vault, &json_path, OnConflict::Skip).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("json import"));
        assert!(msg.contains("not valid UTF-8"));
        assert!(vault.list().unwrap().is_empty());
    }

    #[test]
    fn imports_json_array() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        let json_path = dir.path().join("prompts.json");
        std::fs::write(
            &json_path,
            r#"[{"title":"A","body":"one","tags":["x"]},{"title":"B","body":"two"}]"#,
        )
        .unwrap();
        let report = import_json_array(&vault, &json_path, OnConflict::Skip).unwrap();
        assert_eq!(report.created, 2);
        assert_eq!(vault.list().unwrap().len(), 2);
    }

    #[test]
    fn normalizes_folder_labels_on_import() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        let json_path = dir.path().join("prompts.json");
        std::fs::write(
            &json_path,
            r#"[{"title":"A","body":"one","folder":"  Writing // Blog Posts / "}]"#,
        )
        .unwrap();
        let report = import_json_array(&vault, &json_path, OnConflict::Skip).unwrap();
        assert_eq!(report.created, 1);
        assert!(report.errors.is_empty());
        let ids = vault.list().unwrap();
        assert_eq!(ids.len(), 1);
        let prompt = vault.read(&ids[0]).unwrap();
        assert_eq!(prompt.fm.folder.as_deref(), Some("Writing/Blog Posts"));
    }

    #[test]
    fn rejects_item_with_traversal_folder() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        let json_path = dir.path().join("prompts.json");
        std::fs::write(
            &json_path,
            r#"[{"title":"Bad","body":"x","folder":"../evil"},{"title":"Good","body":"y"}]"#,
        )
        .unwrap();
        let report = import_json_array(&vault, &json_path, OnConflict::Skip).unwrap();
        assert_eq!(report.created, 1);
        assert_eq!(report.errors.len(), 1);
        let ids = vault.list().unwrap();
        assert_eq!(ids.len(), 1);
        let prompt = vault.read(&ids[0]).unwrap();
        assert_eq!(prompt.fm.title, "Good");
    }

    #[test]
    fn reports_item_with_invalid_id() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        let json_path = dir.path().join("prompts.json");
        std::fs::write(
            &json_path,
            r#"[{"id":"not-a-ulid","title":"Bad"},{"title":"Good","body":"y"}]"#,
        )
        .unwrap();
        let report = import_json_array(&vault, &json_path, OnConflict::Skip).unwrap();
        assert_eq!(report.created, 1);
        assert_eq!(report.errors.len(), 1);
        assert!(report.errors[0].contains("invalid prompt id"));
        let ids = vault.list().unwrap();
        assert_eq!(ids.len(), 1);
        let prompt = vault.read(&ids[0]).unwrap();
        assert_eq!(prompt.fm.title, "Good");
    }

    #[test]
    fn valid_id_still_skips_on_reimport() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        let json_path = dir.path().join("prompts.json");
        std::fs::write(
            &json_path,
            r#"[{"id":"01HXY3K9F2N8PQRSTVWXYZABCD","title":"A"}]"#,
        )
        .unwrap();
        let report1 = import_json_array(&vault, &json_path, OnConflict::Skip).unwrap();
        assert_eq!(report1.created, 1);
        assert!(report1.errors.is_empty());
        let report2 = import_json_array(&vault, &json_path, OnConflict::Skip).unwrap();
        assert_eq!(report2.skipped, 1);
        assert_eq!(report2.created, 0);
        assert_eq!(vault.list().unwrap().len(), 1);
    }

    #[test]
    fn empty_folder_string_imports_as_none() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        let json_path = dir.path().join("prompts.json");
        std::fs::write(&json_path, r#"[{"title":"A","body":"one","folder":""}]"#).unwrap();
        let report = import_json_array(&vault, &json_path, OnConflict::Skip).unwrap();
        assert_eq!(report.created, 1);
        assert!(report.errors.is_empty());
        let ids = vault.list().unwrap();
        assert_eq!(ids.len(), 1);
        let prompt = vault.read(&ids[0]).unwrap();
        assert_eq!(prompt.fm.folder, None);
    }
}
