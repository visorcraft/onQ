use std::path::PathBuf;
use std::sync::mpsc;

use notify::{event::EventKind, RecursiveMode, Watcher};

use crate::error::CoreResult;

pub enum VaultEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Removed(PathBuf),
}

pub struct VaultWatcher {
    _watcher: notify::RecommendedWatcher,
    pub rx: mpsc::Receiver<VaultEvent>,
}

pub fn watch(root: PathBuf) -> CoreResult<VaultWatcher> {
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            let ev = match event.kind {
                EventKind::Create(_) => VaultEvent::Created(event.paths[0].clone()),
                EventKind::Modify(_) => VaultEvent::Modified(event.paths[0].clone()),
                EventKind::Remove(_) => VaultEvent::Removed(event.paths[0].clone()),
                _ => return,
            };
            let _ = tx.send(ev);
        }
    })?;
    watcher.watch(&root, RecursiveMode::Recursive)?;
    Ok(VaultWatcher {
        _watcher: watcher,
        rx,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn detects_create_and_modify() {
        let dir = TempDir::new().unwrap();
        let w = watch(dir.path().to_path_buf()).unwrap();
        std::fs::write(dir.path().join("a.md"), "---\n").unwrap();
        std::fs::write(dir.path().join("a.md"), "---\nupdated").unwrap();
        let first = w.rx.recv_timeout(Duration::from_secs(2));
        assert!(matches!(
            first,
            Ok(VaultEvent::Created(_)) | Ok(VaultEvent::Modified(_))
        ));
    }
}
