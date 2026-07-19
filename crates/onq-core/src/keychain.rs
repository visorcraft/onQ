#[cfg(feature = "real-keychain")]
use crate::error::CoreError;
use crate::error::CoreResult;

pub const SERVICE: &str = "onQ";

pub trait Keychain: Send + Sync {
    fn get(&self, key: &str) -> CoreResult<Option<Vec<u8>>>;
    fn set(&self, key: &str, value: &[u8]) -> CoreResult<()>;
    fn delete(&self, key: &str) -> CoreResult<()>;
}

#[cfg(feature = "real-keychain")]
pub struct OsKeychain;

#[cfg(feature = "real-keychain")]
impl Keychain for OsKeychain {
    fn get(&self, key: &str) -> CoreResult<Option<Vec<u8>>> {
        let entry =
            keyring::Entry::new(SERVICE, key).map_err(|e| CoreError::Keychain(e.to_string()))?;
        match entry.get_password() {
            Ok(s) => Ok(Some(s.into_bytes())),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(CoreError::Keychain(e.to_string())),
        }
    }
    fn set(&self, key: &str, value: &[u8]) -> CoreResult<()> {
        let s = std::str::from_utf8(value).map_err(|e| CoreError::Encryption(e.to_string()))?;
        let entry =
            keyring::Entry::new(SERVICE, key).map_err(|e| CoreError::Keychain(e.to_string()))?;
        entry
            .set_password(s)
            .map_err(|e| CoreError::Keychain(e.to_string()))
    }
    fn delete(&self, key: &str) -> CoreResult<()> {
        let entry =
            keyring::Entry::new(SERVICE, key).map_err(|e| CoreError::Keychain(e.to_string()))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(CoreError::Keychain(e.to_string())),
        }
    }
}
