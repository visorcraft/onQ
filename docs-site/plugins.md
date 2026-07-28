---
title: Plugin authoring
description: Current plugin package format, signature policy, ABI exports, host capability status, testing, and safety boundaries.
---

# Plugin authoring

onQ plugins are signed Rust native libraries over C ABI version 1. The
packaging and loading path exists, but the host callback surface is not yet
complete. Read [current runtime limits](#current-runtime-limits) before
building.

## Security model

An installable package must:

- be a gzip-compressed tar archive;
- contain `manifest.toml`;
- contain a raw 64-byte ed25519 `signature.sig`;
- sign the exact `manifest.toml` bytes;
- verify against the public trust anchor compiled into onQ;
- declare `GPL-3.0-only`;
- contain a compatible `.so`, `.dylib`, or `.dll`.

Native plugins share onQ's process privileges. Signature verification proves
that a trusted key signed the manifest. It does not sandbox code.

## Archive layout

```text
manifest.toml
signature.sig
plugin.so            # Linux example
```

The installer discovers common native names and then any library extension in
the extracted root. It does not currently use an `[entrypoint]` manifest
table.

Although the file picker accepts `.onqplugin`, `.tar`, `.gz`, and `.tgz`, the
installer always expects gzip-compressed tar bytes.

## Manifest

```toml
[plugin]
id = "com.example.onq.hello"
name = "Hello"
version = "0.1.0"
license = "GPL-3.0-only"
capabilities = ["command:hello:Say hello"]
```

Required:

- `plugin.id`
- `plugin.name`
- `plugin.version`
- `plugin.license`

`plugin.capabilities` is optional and defaults to an empty array.

Capability strings currently recognized by host-side UI logic include:

```text
command
commands
command:<id>:<display name>
embedding
embedder
embedding...
```

Capability-declared commands can appear in the Palette. They do not run
plugin business logic in the current host.

## Minimal Rust library

Inside this workspace, depend on the SDK:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
onq-plugin-sdk = { path = "../../../crates/onq-plugin-sdk" }
```

Export ABI metadata and initialization:

```rust
use onq_plugin_sdk::{export_plugin, HostApi};

fn init(_api: *const HostApi) -> i32 {
    0
}

export_plugin!("Hello", "0.1.0", "[]", init);
```

The macro exports:

```text
onq_plugin_abi_version
onq_plugin_name
onq_plugin_version
onq_plugin_capabilities
onq_plugin_init
```

`onq_plugin_capabilities` must return JSON text. The manifest capabilities and
library-exported capabilities are currently separate values.

## HostApi ABI

The SDK declares this function table:

| Callback | Intended role | Current host behavior |
|---|---|---|
| `db_query` | Query host database | Returns `-1` |
| `db_execute` | Execute database mutation | Returns `-1` |
| `vault_read` | Read vault data | Returns `-1` |
| `vault_write` | Write vault data | Returns `-1` |
| `keychain_get` | Read scoped key | Returns `-1` |
| `keychain_set` | Write scoped key | Returns `-1` |
| `embedding_embed` | Use active embedder | Returns `-1` |
| `register_command` | Register Palette command metadata | Implemented |
| `log` | Host logging | No-op |
| `config_get` | Read plugin config | Returns `-1` |
| `config_set` | Write plugin config | Returns `-1` |

The table version is `1`.

## Current runtime limits

The host:

1. verifies and extracts a package;
2. records it in the per-vault plugin table;
3. finds and loads its native library;
4. verifies ABI version;
5. reads exported name, version, and JSON capabilities;
6. calls `onq_plugin_init`;
7. records commands registered through `register_command`;
8. shows enabled commands in the Palette.

It does not call the registered handler pointer when a Palette command is
selected. The UI receives a diagnostic success string instead. Most other
HostApi callbacks are stubs as listed above.

Do not publish a plugin that depends on those callbacks until the host
implementation and command-handler ABI are complete.

## Sign and package

Generate an ed25519 key and sign exact manifest bytes:

```bash
openssl genpkey -algorithm ed25519 -out signing-key.pem
openssl pkeyutl -sign \
  -inkey signing-key.pem \
  -in manifest.toml \
  -out signature.sig
```

Verify locally:

```bash
openssl pkeyutl -verify \
  -inkey signing-key.pem \
  -in manifest.toml \
  -sigfile signature.sig
```

Package, using the actual target library name:

```bash
tar czf hello.onqplugin manifest.toml signature.sig plugin.so
```

::: warning
A self-signed package does not install in a stock onQ build unless its public
key matches the compiled trust anchor. The sample fixture uses a test-only
trust-anchor override. Production distribution requires coordination with
the onQ maintainers. There is no user-managed trust store.
:::

Never commit a production signing private key. The repository fixture key is
test data only.

## Install and manage

1. Open an unlocked vault.
2. Open **Settings > Plugins**.
3. Select **Install** and choose the archive.
4. onQ verifies, extracts, records, enables, and initializes it.

Installed files live at:

```text
<vault>/.onq/plugins/installed/<plugin-id>/
```

The Settings page can disable, re-enable, refresh, or uninstall a plugin.
Uninstall removes the registry row and installed directory.

Duplicate IDs are rejected instead of overwritten.

## Tests and sample

The buildable fixture lives at:

```text
tests/fixtures/sample-plugin/
```

Build and sign it:

```bash
cd tests/fixtures/sample-plugin
bash build-and-sign.sh
```

Run workspace plugin coverage:

```bash
cargo test -p onq-core
cargo test -p onq-app
cargo test -p onq-plugin-sdk
```

The sample proves native exports, signature layout, archive extraction, and
initialization. It does not change the stock app trust anchor.

## Compatibility rules

- Host and plugin ABI versions must match.
- Plugins are platform- and architecture-specific native binaries.
- A Linux `.so` does not run on Windows or macOS.
- Manifest license must be exactly `GPL-3.0-only`.
- Archive paths must remain inside the package root.
- Existing plugin IDs must be uninstalled before reinstall.

The repository-level authoring source is
[`docs/plugins/README.md`](https://github.com/visorcraft/onQ/blob/master/docs/plugins/README.md).
