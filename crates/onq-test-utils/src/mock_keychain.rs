use std::collections::HashMap;
use std::sync::Mutex;

use onq_core::error::CoreResult;
use onq_core::keychain::Keychain;

pub struct MockKeychain {
    store: Mutex<HashMap<String, Vec<u8>>>,
}

impl MockKeychain {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MockKeychain {
    fn default() -> Self {
        Self::new()
    }
}

impl Keychain for MockKeychain {
    fn get(&self, key: &str) -> CoreResult<Option<Vec<u8>>> {
        Ok(self.store.lock().unwrap().get(key).cloned())
    }
    fn set(&self, key: &str, value: &[u8]) -> CoreResult<()> {
        self.store
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_vec());
        Ok(())
    }
    fn delete(&self, key: &str) -> CoreResult<()> {
        self.store.lock().unwrap().remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_delete() {
        let k = MockKeychain::new();
        assert_eq!(k.get("absent").unwrap(), None);
        k.set("k", b"v").unwrap();
        assert_eq!(k.get("k").unwrap(), Some(b"v".to_vec()));
        k.delete("k").unwrap();
        assert_eq!(k.get("k").unwrap(), None);
    }

    #[test]
    fn default_works() {
        let k = MockKeychain::default();
        k.set("x", b"y").unwrap();
        assert_eq!(k.get("x").unwrap(), Some(b"y".to_vec()));
    }
}
