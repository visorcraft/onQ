use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml parse: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("json parse: {0}")]
    Json(#[from] serde_json::Error),
    #[error("toml parse: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("file not found: {0}")]
    NotFound(PathBuf),
    #[error("invalid frontmatter: {0}")]
    Frontmatter(String),
    #[error("invalid id: {0}")]
    Id(#[from] crate::ulid::IdError),
    #[error("notify: {0}")]
    Notify(#[from] notify::Error),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("keychain: {0}")]
    Keychain(String),
    #[error("encryption: {0}")]
    Encryption(String),
    #[error("merge: {0}")]
    Merge(String),
    #[error("db: {0}")]
    Db(String),
    #[error("plugin: {0}")]
    Plugin(String),
    #[error("other: {0}")]
    Other(String),
}

pub type CoreResult<T> = std::result::Result<T, CoreError>;
