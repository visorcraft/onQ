# Plugin authoring guide

onQ loads **signed** Rust-native plugins via a versioned C ABI (`onq-plugin-sdk`).

## Requirements

- License must be **GPL-3.0-only** in the plugin manifest
- Manifest must be ed25519-signed with a key trusted by the project
- ABI version currently: **1** (`onq_plugin_abi_version`)

## Manifest (`plugin.toml`)

```toml
[plugin]
id = "example.hello"
name = "Hello"
version = "0.1.0"
license = "GPL-3.0-only"
capabilities = ["read"]
```

## Minimal plugin (Rust)

```rust
use onq_plugin_sdk::{export_plugin, HostApi};

fn init(_api: *const HostApi) -> i32 {
    0
}

export_plugin!("example.hello", "0.1.0", "read", init);
```

See `tests/fixtures/sample-plugin/` for a buildable sample.

## Install

1. Build a plugin archive (`.onqplugin` / tar layout expected by `plugin_install`)
2. Settings → Plugins → Install… (or `install_plugin` Tauri command)
3. Enable the plugin; unsigned/tampered packages are quarantined

## Host API surface

`HostApi` exposes vault read/write helpers, keychain get/set, embedding embed,
command registration, and config get/set. See `crates/onq-plugin-sdk/src/lib.rs`.

## Alternate embedders

Plugins that advertise an embedding capability can be selected as the active
embedder once the host embedder hook is enabled (Settings → Search). The
built-in default remains `sentence-transformers/all-MiniLM-L6-v2`.
