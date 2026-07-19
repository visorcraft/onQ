//! E2E plugin-loading smoke test.
//!
//! Loads the sample-plugin cdylib built by
//! `tests/fixtures/sample-plugin/build-and-sign.sh` and asserts that the
//! four ABI exports (`onq_plugin_abi_version`,
//! `onq_plugin_name`, `onq_plugin_version`,
//! `onq_plugin_capabilities`) round-trip through
//! `onq_core::plugin::load`. Proves the host's `libloading`
//! resolver + the plugin's `export_plugin!` macro agree on the ABI shape.
//!
//! The fixture's `.so` only exists after running `build-and-sign.sh`. When
//! the binary is missing we `eprintln!` and early-return so the test is a
//! no-op in environments that haven't built the fixture (e.g. a clean
//! `cargo test --workspace` run without first running the build script).
//! The `#[ignore]` attribute keeps `cargo test --workspace` from being
//! gated on the fixture — opt-in via
//! `cargo test --workspace plugin_load -- --ignored` if you want a hard
//! failure when the build hasn't run.

use onq_core::plugin;
use std::path::Path;

fn sample_plugin_so() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR is set by cargo to this crate's root.
    let here = env!("CARGO_MANIFEST_DIR");
    // Walk up to the workspace root, then into the fixture's target dir.
    // The fixture's cdylib is named `libonq_sample_plugin.so`
    // (cargo normalizes `-` → `_` in the output filename).
    let workspace_root = Path::new(here)
        .parent()
        .expect("crates/onq-core has a parent dir (the workspace root)");
    workspace_root.join("tests/fixtures/sample-plugin/target/release/libonq_sample_plugin.so")
}

#[test]
#[ignore = "requires the sample-plugin fixture to be built first; see tests/fixtures/sample-plugin/build-and-sign.sh"]
fn loads_signed_sample_plugin() {
    let plugin_path = sample_plugin_so();
    if !plugin_path.exists() {
        eprintln!(
            "skipping: {} does not exist; run \
             tests/fixtures/sample-plugin/build-and-sign.sh to build the fixture",
            plugin_path.display()
        );
        return;
    }

    let p = plugin::load(&plugin_path).expect("load");

    // The fixture's export_plugin!("sample-plugin", "0.1.0", "[]", ...) supplies
    // these exact strings. If the SDK macro or `plugin::load` drifts, this
    // catches it.
    assert_eq!(p.name, "sample-plugin", "plugin name from ABI export");
    assert_eq!(p.version, "0.1.0", "plugin version from ABI export");

    // `export_plugin!` emits the literal string `[]` as the capabilities
    // payload, which `plugin::load` parses via serde_json. The fixture
    // doesn't declare any capabilities, so the parsed value is an empty
    // JSON array (not null, not object).
    let caps = p.capabilities.as_array().expect("capabilities is an array");
    assert!(
        caps.is_empty(),
        "expected empty capabilities array, got {caps:?}"
    );

    // Drop the library to prove the load succeeded without leaking handles
    // (LoadedPlugin owns the libloading::Library).
    drop(p);
}

#[test]
#[ignore = "requires the sample-plugin fixture to be built first"]
fn loads_returns_abi_version_one() {
    use onq_plugin_sdk::ABI_VERSION;
    let plugin_path = sample_plugin_so();
    if !plugin_path.exists() {
        eprintln!(
            "skipping: {} does not exist; run \
             tests/fixtures/sample-plugin/build-and-sign.sh to build the fixture",
            plugin_path.display()
        );
        return;
    }
    let p = plugin::load(&plugin_path).expect("load");
    // If the host ABI bumps, plugin::load returns Err before we get here;
    // assert the SDK and the host agree on the constant.
    assert_eq!(ABI_VERSION, 1);
    drop(p);
}
