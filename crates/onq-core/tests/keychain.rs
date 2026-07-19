//! Integration tests for the Keychain trait, using the MockKeychain from
//! onq-test-utils to exercise trait semantics without touching the
//! real OS keyring.

use onq_core::keychain::Keychain;
use onq_test_utils::mock_keychain::MockKeychain;

#[test]
fn roundtrip() {
    let k = MockKeychain::new();
    k.set("master", b"hunter2").unwrap();
    assert_eq!(k.get("master").unwrap(), Some(b"hunter2".to_vec()));
    k.delete("master").unwrap();
    assert_eq!(k.get("master").unwrap(), None);
}

#[test]
fn overwrite() {
    let k = MockKeychain::new();
    k.set("k", b"v1").unwrap();
    k.set("k", b"v2").unwrap();
    assert_eq!(k.get("k").unwrap(), Some(b"v2".to_vec()));
}

#[test]
fn missing_returns_none() {
    let k = MockKeychain::new();
    assert_eq!(k.get("nope").unwrap(), None);
}
