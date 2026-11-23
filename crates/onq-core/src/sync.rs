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

/// Maps a notify event to a [`VaultEvent`], or `None` for events we do not handle.
///
/// Empty-path events are dropped: OS backends can emit events with no paths
/// (e.g. placeholder or renamed-handle events), and the old code panicked on
/// indexing `paths[0]`.
fn map_notify_event(event: notify::Event) -> Option<VaultEvent> {
    let path = event.paths.first()?.clone();
    let ev = match event.kind {
        EventKind::Create(_) => VaultEvent::Created(path),
        EventKind::Modify(_) => VaultEvent::Modified(path),
        EventKind::Remove(_) => VaultEvent::Removed(path),
        _ => return None,
    };
    Some(ev)
}

pub fn watch(root: PathBuf) -> CoreResult<VaultWatcher> {
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            if let Some(ev) = map_notify_event(event) {
                let _ = tx.send(ev);
            }
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
    use notify::event::{AccessKind, CreateKind, ModifyKind, RemoveKind};
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

    #[test]
    fn maps_create_modify_remove_kinds() {
        let path = PathBuf::from("/tmp/x.md");

        let created =
            notify::Event::new(EventKind::Create(CreateKind::File)).add_path(path.clone());
        assert!(matches!(
            map_notify_event(created),
            Some(VaultEvent::Created(p)) if p == path
        ));

        let modified =
            notify::Event::new(EventKind::Modify(ModifyKind::Any)).add_path(path.clone());
        assert!(matches!(
            map_notify_event(modified),
            Some(VaultEvent::Modified(p)) if p == path
        ));

        let removed =
            notify::Event::new(EventKind::Remove(RemoveKind::File)).add_path(path.clone());
        assert!(matches!(
            map_notify_event(removed),
            Some(VaultEvent::Removed(p)) if p == path
        ));
    }

    #[test]
    fn empty_paths_events_are_dropped() {
        let created = notify::Event::new(EventKind::Create(CreateKind::File));
        assert!(map_notify_event(created).is_none());

        let modified = notify::Event::new(EventKind::Modify(ModifyKind::Any));
        assert!(map_notify_event(modified).is_none());

        let removed = notify::Event::new(EventKind::Remove(RemoveKind::File));
        assert!(map_notify_event(removed).is_none());
    }

    #[test]
    fn other_event_kinds_are_ignored() {
        let path = PathBuf::from("/tmp/x.md");

        let access = notify::Event::new(EventKind::Access(AccessKind::Any)).add_path(path.clone());
        assert!(map_notify_event(access).is_none());

        let other = notify::Event::new(EventKind::Other).add_path(path);
        assert!(map_notify_event(other).is_none());
    }
}
