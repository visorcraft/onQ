use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PromptId(String);

impl Serialize for PromptId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for PromptId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        PromptId::from_string(s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

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
    pub fn is_valid_str(s: &str) -> bool {
        s.len() == 26 && s.bytes().all(|b| b.is_ascii_alphanumeric())
    }

    pub fn from_string(s: String) -> Result<Self, IdError> {
        if Self::is_valid_str(&s) {
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
    #[test]
    fn serde_valid_id_roundtrips() {
        let id: PromptId = serde_json::from_str(r#""01HXY3K9F2N8PQRSTVWXYZABCD""#).unwrap();
        assert_eq!(id.as_str(), "01HXY3K9F2N8PQRSTVWXYZABCD");
        let serialized = serde_json::to_string(&id).unwrap();
        let back: PromptId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(back, id);
    }
    #[test]
    fn serde_rejects_invalid_ids() {
        let err = serde_json::from_str::<PromptId>(r#""../../evil""#).unwrap_err();
        assert!(err.to_string().contains("invalid prompt id"));
        let err = serde_json::from_str::<PromptId>(r#""short""#).unwrap_err();
        assert!(err.to_string().contains("invalid prompt id"));
    }
}
