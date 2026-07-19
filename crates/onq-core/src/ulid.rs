use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PromptId(String);

fn monotonic_generator() -> &'static Mutex<ulid::Generator> {
    static GEN: OnceLock<Mutex<ulid::Generator>> = OnceLock::new();
    GEN.get_or_init(|| Mutex::new(ulid::Generator::new()))
}

impl PromptId {
    pub fn new() -> Self {
        let mut gen = monotonic_generator()
            .lock()
            .expect("ulid generator poisoned");
        let ulid = gen.generate().expect("monotonic ulid overflow").to_string();
        Self(ulid)
    }
    pub fn from_string(s: String) -> Result<Self, IdError> {
        if s.len() == 26 && s.chars().all(|c| c.is_ascii_alphanumeric()) {
            Ok(Self(s))
        } else {
            Err(IdError::Invalid(s))
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for PromptId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PromptId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IdError {
    #[error("invalid prompt id: {0}")]
    Invalid(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_is_26_chars() {
        let id = PromptId::new();
        assert_eq!(id.as_str().len(), 26);
    }
    #[test]
    fn from_string_rejects_garbage() {
        assert!(PromptId::from_string("nope".into()).is_err());
        assert!(PromptId::from_string("a".repeat(27)).is_err());
    }
    #[test]
    fn ids_are_unique() {
        let a = PromptId::new();
        let b = PromptId::new();
        assert_ne!(a, b);
    }
    #[test]
    fn ids_sort_lexicographically() {
        let a = PromptId::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = PromptId::new();
        assert!(a.as_str() < b.as_str());
    }
}
