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
        let raw = std::fs::read_to_string(&path).map_err(|_| CoreError::NotFound(path.clone()))?;
        let (fm, body) = frontmatter::parse(&raw)?;
        Ok(Prompt { fm, body })
    }

    pub fn write(&self, prompt: &Prompt) -> CoreResult<()> {
        let path = self.prompt_path(&prompt.fm.id);
        let rendered = frontmatter::render(&prompt.fm, &prompt.body);
        history::snapshot(self, &prompt.fm.id, &prompt.body)?;
        atomic_write(&path, rendered.as_bytes())?;
        history::prune_older_than(self, 30)?;
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
}
