//! Domain logic for onQ. No Tauri dependencies.

pub mod audit;
pub mod backup;
pub mod crypto;
pub mod db;
pub mod embed;
pub mod embedding_index;
pub mod error;
pub mod export;
pub mod folder_path;
pub mod frontmatter;
pub mod history;
pub mod import;
pub mod keychain;
pub mod lock;
pub mod merge;
pub mod migrate;
pub mod path_util;
pub mod plugin;
pub mod plugin_install;
pub mod reconcile;
pub mod recovery;
pub mod schema;
pub mod search;
pub mod signature;
pub mod smart_folder_dsl;
pub mod smart_folder_visual;
pub mod sync;
pub mod sync_state;
pub mod tag_suggest;
pub mod template;
pub mod ulid;
pub mod vault;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_semver() {
        let v = version();
        assert!(v.split('.').count() >= 2, "version {} not semver", v);
    }
}
