use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};
use crate::frontmatter::{self, Frontmatter};
use crate::history;
use crate::ulid::PromptId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub fm: Frontmatter,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct Vault {
    pub root: PathBuf,
}

impl Vault {
    pub fn new(root: impl Into<PathBuf>) -> CoreResult<Self> {
        let root = root.into();
        std::fs::create_dir_all(root.join("prompts"))?;
        std::fs::create_dir_all(root.join(".onq"))?;
        Ok(Self { root })
    }

    pub fn prompt_path(&self, id: &PromptId) -> PathBuf {
        self.root
            .join("prompts")
            .join(format!("{}.md", id.as_str()))
    }

    pub fn read(&self, id: &PromptId) -> CoreResult<Prompt> {
        let path = self.prompt_path(id);
        let raw = std::fs::read_to_string(&path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => CoreError::NotFound(path.clone()),
            _ => CoreError::Io(e),
        })?;
        let (fm, body) = frontmatter::parse(&raw)?;
        Ok(Prompt { fm, body })
    }

    pub fn write(&self, prompt: &Prompt) -> CoreResult<()> {
        self.write_with_history_retention(prompt, 30)
    }

    /// Write prompt body/frontmatter and snapshot history, pruning snapshots
    /// older than `history_retention_days` (0 removes all previous snapshots
    /// after the new write's snapshot is taken).
    pub fn write_with_history_retention(
        &self,
        prompt: &Prompt,
        history_retention_days: u32,
    ) -> CoreResult<()> {
        if !PromptId::is_valid_str(prompt.fm.id.as_str()) {
            return Err(CoreError::Other(format!(
                "invalid prompt id: {}",
                prompt.fm.id.as_str()
            )));
        }
        let path = self.prompt_path(&prompt.fm.id);
        let rendered = frontmatter::render(&prompt.fm, &prompt.body);
        history::snapshot(self, &prompt.fm.id, &prompt.body)?;
        atomic_write(&path, rendered.as_bytes())?;
        history::prune_older_than(self, history_retention_days)?;
        Ok(())
    }

    pub fn list(&self) -> CoreResult<Vec<PromptId>> {
        let dir = self.root.join("prompts");
        let mut ids = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(id_str) = name.strip_suffix(".md") {
                if let Ok(id) = PromptId::from_string(id_str.to_string()) {
                    ids.push(id);
                }
            }
        }
        ids.sort();
        Ok(ids)
    }

    pub fn delete(&self, id: &PromptId) -> CoreResult<()> {
        let path = self.prompt_path(id);
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn new_prompt(&self, title: impl Into<String>) -> CoreResult<Prompt> {
        let now = Utc::now();
        let fm = Frontmatter {
            id: PromptId::new(),
            title: title.into(),
            folder: None,
            tags: vec![],
            favorite: false,
            locked: false,
            created: now,
            updated: now,
        };
        Ok(Prompt {
            fm,
            body: String::new(),
        })
    }
}

pub(crate) fn atomic_write(path: &Path, data: &[u8]) -> CoreResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn roundtrip() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let mut p = vault.new_prompt("Test").unwrap();
        p.body = "Hello world".into();
        vault.write(&p).unwrap();
        let read = vault.read(&p.fm.id).unwrap();
        assert_eq!(read.fm.title, "Test");
        assert_eq!(read.body, "Hello world");
    }
    #[test]
    fn list_returns_sorted() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let a = vault.new_prompt("A").unwrap();
        let b = vault.new_prompt("B").unwrap();
        vault.write(&a).unwrap();
        vault.write(&b).unwrap();
        let ids = vault.list().unwrap();
        assert_eq!(ids, vec![a.fm.id.clone(), b.fm.id.clone()]);
    }
    #[test]
    fn delete_then_read_not_found() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let p = vault.new_prompt("X").unwrap();
        vault.write(&p).unwrap();
        vault.delete(&p.fm.id).unwrap();
        assert!(matches!(vault.read(&p.fm.id), Err(CoreError::NotFound(_))));
    }
    #[test]
    fn read_missing_prompt_is_not_found() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let id = PromptId::new();
        assert!(matches!(vault.read(&id), Err(CoreError::NotFound(_))));
    }
    #[test]
    fn read_corrupt_non_utf8_file_is_io_error_not_not_found() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let id = PromptId::new();
        std::fs::write(vault.prompt_path(&id), b"\xff\xfe\x00bad").unwrap();
        let err = vault.read(&id).unwrap_err();
        assert!(!matches!(err, CoreError::NotFound(_)));
        assert!(matches!(err, CoreError::Io(_)));
        assert!(err.to_string().contains("io error"));
    }
    #[test]
    fn read_unreadable_directory_path_is_not_not_found() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let id = PromptId::new();
        std::fs::create_dir_all(vault.prompt_path(&id)).unwrap();
        let err = vault.read(&id).unwrap_err();
        assert!(!matches!(err, CoreError::NotFound(_)));
    }
    #[test]
    fn frontmatter_parse_rejects_invalid_id() {
        let raw = "---\nid: ../../evil\ntitle: T\ncreated: 2026-07-18T10:30:00Z\nupdated: 2026-07-18T10:30:00Z\n---\n\nbody";
        assert!(frontmatter::parse(raw).is_err());
    }
    #[test]
    fn is_valid_str_boundaries() {
        assert!(PromptId::is_valid_str("01HXY3K9F2N8PQRSTVWXYZABCD"));
        assert!(!PromptId::is_valid_str("01HXY3K9F2N8PQRSTVWXYZABC")); // 25
        assert!(!PromptId::is_valid_str("01HXY3K9F2N8PQRSTVWXYZABCDE")); // 27
        assert!(!PromptId::is_valid_str("01HXY3K9F2N8PQRSTVWXYZABC/")); // 26, bad char
        assert!(!PromptId::is_valid_str("01HXY3K9F2N8PQRSTVWXYZABC.")); // 26, bad char
        assert!(!PromptId::is_valid_str("01HXY3K9F2N8PQRSTVWXYZABé")); // 26 bytes, non-ASCII
        assert!(!PromptId::is_valid_str(""));
    }
    #[test]
    fn write_guard_rejects_invalid_id() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path().join("vault")).unwrap();
        let res: Result<Prompt, _> = serde_json::from_str(
            r#"{"fm":{"id":"../../evil","title":"T","created":"2026-07-18T10:30:00Z","updated":"2026-07-18T10:30:00Z"},"body":"b"}"#,
        );
        assert!(res.is_err());
        let p = vault.new_prompt("Test").unwrap();
        vault.write(&p).unwrap();
        assert!(!dir.path().join("evil.md").exists());
    }
    #[test]
    fn read_malformed_frontmatter_still_frontmatter_error() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        let id = PromptId::new();
        std::fs::write(vault.prompt_path(&id), b"not yaml at all").unwrap();
        assert!(matches!(vault.read(&id), Err(CoreError::Frontmatter(_))));
    }
}
