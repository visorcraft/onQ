# Plugin authoring

onQ has a signed native-plugin package and C ABI. Packaging, signature
verification, loading, and initialization work. Most host callbacks and
registered command execution remain incomplete.

Read the public [plugin guide](https://visorcraft.github.io/onQ/plugins) and
[feature status](https://visorcraft.github.io/onQ/feature-status) before
building against the current 1.x API.

## Requirements

- Rust native library using C ABI version `1`
- `GPL-3.0-only` manifest license
- gzip-compressed tar package
- exact root files `manifest.toml` and `signature.sig`
- raw 64-byte ed25519 signature over the exact `manifest.toml` bytes
- signing key matching the trust anchor compiled into onQ
- platform-specific `.so`, `.dylib`, or `.dll`

Plugins run with the onQ process privileges. The signature verifies the
signer and manifest. It does not sandbox native code.

## Package layout

```text
manifest.toml
signature.sig
plugin.so
```

The Settings picker accepts `.onqplugin`, `.tar`, `.gz`, and `.tgz`, but the
installer always decodes gzip-compressed tar content. Files must be at the
archive root. Package paths that escape the staging directory are rejected.

## Manifest

```toml
[plugin]
id = "com.example.onq.hello"
name = "Hello"
version = "0.1.0"
license = "GPL-3.0-only"
capabilities = ["command:hello:Say hello"]
```

`id`, `name`, `version`, and `license` are required.
`capabilities` is optional and defaults to an empty array. The installer does
not currently use an `[entrypoint]` table.

Capability strings understood by current host UI logic include:

```text
command
commands
command:<id>:<display name>
embedding
embedder
embedding...
```

Capability strings are discovery metadata. They do not grant access or make
an incomplete host callback operational.

## Minimal Rust library

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
onq-plugin-sdk = { path = "../../../crates/onq-plugin-sdk" }
```

```rust
use onq_plugin_sdk::{export_plugin, HostApi};

fn init(_api: *const HostApi) -> i32 {
    0
}

export_plugin!("Hello", "0.1.0", "[]", init);
```

`export_plugin!` exports:

```text
onq_plugin_abi_version
onq_plugin_name
onq_plugin_version
onq_plugin_capabilities
onq_plugin_init
```

The exported capabilities value must be JSON text. Manifest capabilities and
library-exported capabilities are currently separate.

## HostApi status

| Callback | Current host behavior |
|---|---|
| `db_query` | Returns `-1` |
| `db_execute` | Returns `-1` |
| `vault_read` | Returns `-1` |
| `vault_write` | Returns `-1` |
| `keychain_get` | Returns `-1` |
| `keychain_set` | Returns `-1` |
| `embedding_embed` | Returns `-1` |
| `register_command` | Records Palette command metadata |
| `log` | No-op |
| `config_get` | Returns `-1` |
| `config_set` | Returns `-1` |

The host records handler pointers passed to `register_command`, but it does
not invoke them when a Palette command is selected. The current result is a
diagnostic success string.

Treat this as a packaging and ABI preview. Do not publish a plugin that
depends on incomplete callbacks.

## Sign and package

Generate a development key:

```bash
openssl genpkey -algorithm ed25519 -out signing-key.pem
```

Sign the unmodified manifest:

```bash
openssl pkeyutl -sign \
  -inkey signing-key.pem \
  -in manifest.toml \
  -out signature.sig
```

Verify it:

```bash
openssl pkeyutl -verify \
  -inkey signing-key.pem \
  -in manifest.toml \
  -sigfile signature.sig
```

Create the package:

```bash
tar czf hello.onqplugin manifest.toml signature.sig plugin.so
```

A self-signed package does not install in a stock onQ build unless its public
key matches the compiled trust anchor. Production signing requires
coordination with the onQ maintainers. There is no user-managed trust store.
Never commit a production signing private key.

## Install lifecycle

1. Open an unlocked vault.
2. Open **Settings > Plugins**.
3. Select **Install** and choose the package.
4. onQ verifies the manifest, signature, and license.
5. onQ extracts files to `.onq/plugins/installed/<plugin-id>/`.
6. onQ records, enables, loads, and initializes the plugin.

Duplicate plugin IDs are rejected. Uninstall removes the registry entry and
installed directory.

## Sample and tests

The workspace fixture is under `tests/fixtures/sample-plugin/`.

```bash
cd tests/fixtures/sample-plugin
bash build-and-sign.sh
```

It demonstrates the native exports, archive format, signature, and
initialization. The test-only trust-anchor override does not change a stock
app build.

Run plugin coverage from the repository root:

```bash
cargo test -p onq-core
cargo test -p onq-app
cargo test -p onq-plugin-sdk
```

## Compatibility

- Host and plugin ABI versions must match.
- Native libraries are platform- and architecture-specific.
- Manifest license must be exactly `GPL-3.0-only`.
- Existing IDs must be uninstalled before reinstall.
- Any change to `manifest.toml` invalidates its signature.
- ABI version `1` is still incomplete and may evolve with the 1.x line.
