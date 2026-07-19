//! Integration test — `vault.write` archives the prior body to history
//! before overwriting the prompt file.
//!
//! Proves the M4.5 wiring requirement: every write to a prompt produces a
//! snapshot of the body that was about to be replaced, retrievable via
//! `list_snapshots` and the snapshot file's contents.

use onq_core::history;
use onq_core::vault::Vault;
use tempfile::TempDir;

#[test]
fn write_archives_prior_body_to_history() {
    let dir = TempDir::new().unwrap();
    let vault = Vault::new(dir.path()).unwrap();
    let mut p = vault.new_prompt("Snapshots").unwrap();

    // First write: snapshot of "first body", then the file holds "first body".
    p.body = "first body".into();
    vault.write(&p).unwrap();

    // Second write: snapshot of "second body" (about to be on disk), then the
    // file holds "second body". The prior write's body ("first body") is now
    // recoverable only from the earlier snapshot.
    p.body = "second body".into();
    vault.write(&p).unwrap();

    let snaps = history::list_snapshots(&vault, &p.fm.id).unwrap();
    assert!(
        snaps.len() >= 2,
        "expected at least two snapshots after two writes, got {}",
        snaps.len()
    );

    // The first (oldest) snapshot holds the body of the first write.
    let archived = std::fs::read_to_string(&snaps[0]).unwrap();
    assert_eq!(
        archived, "first body",
        "oldest snapshot must contain the body of the first write"
    );

    // And the live file now holds the newest body.
    let live = vault.read(&p.fm.id).unwrap();
    assert_eq!(live.body, "second body");
}
