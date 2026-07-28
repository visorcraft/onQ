---
title: Installation
description: Supported release targets, runtime requirements, source builds, and first-launch verification.
---

# Installation

Download a release artifact or build onQ from source. The current release
workflow publishes one CPU target per desktop operating system.

## Published targets

| Platform | Release target | Typical bundles |
|---|---|---|
| Windows | `x86_64-pc-windows-msvc` | `.exe` (NSIS), `.msi` |
| macOS | `aarch64-apple-darwin` | `.dmg`, `.app.tar.gz` or `.app.zip` |
| Linux | `x86_64-unknown-linux-gnu` | `.AppImage`, `.deb`, `.rpm` |

The automated release does not currently publish Intel macOS or ARM Linux
artifacts. Those platforms may still build from source when all Rust and native
dependencies support the target.

## Runtime requirements

- **Windows:** WebView2. It is included with current Windows releases. Install
  Microsoft's Evergreen WebView2 Runtime if onQ reports that it is missing.
- **macOS:** the system WebKit runtime.
- **Linux:** WebKitGTK 4.1, a system tray implementation, and the usual GTK
  runtime libraries. Exact package names differ by distribution.

The [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) are the
authoritative native dependency list.

## Install a release

1. Open the [GitHub releases page](https://github.com/visorcraft/onQ/releases).
2. Select the newest stable `v1.x` release.
3. Download the artifact matching the target table above.
4. Install or extract it using the normal platform workflow.
5. Launch onQ and choose **Create new vault** or **Open existing vault**.

Release bundles and updater metadata are produced by
`.github/workflows/release.yml`. Do not install assets from the special
`updater` release. That release only hosts the stable `latest.json` feed.

## Build from source

You need:

- Node.js 24 or newer
- Rust stable, using `rust-toolchain.toml`
- Git
- the native Tauri dependencies for your platform

Clone and install:

```bash
git clone https://github.com/visorcraft/onQ
cd onQ
npm install
```

The encrypted search index depends on
[`mongreldb-core`](https://crates.io/crates/mongreldb-core) from crates.io
(declared in the workspace `Cargo.toml`). No sibling checkout is needed.

Run the desktop app:

```bash
npm run dev:app
```

This starts Vite and the Tauri shell. It does not download the optional
embedding model.

Build a production bundle:

```bash
npm run build:app
```

Bundles land under `target/release/bundle/`, or under
`target/<triple>/release/bundle/` when an explicit target is used.

For frontend-only work:

```bash
npm run dev
```

The browser build can render the Svelte UI, but Tauri IPC actions require the
desktop shell.

## First-launch verification

1. Create a temporary vault.
2. Save the recovery phrase if you choose the no-password path.
3. Create and save one prompt.
4. Return home and press **Win+Q**, **Meta+Q**, or **⌘+Q**.
5. Type part of the title and select the result to copy its body.

That verifies the vault, sparse keyword index, palette, and clipboard path.
Semantic search is separate. Enable it from **Settings > Search > Load MiniLM
model**, then use **Refresh status** to confirm that the model is loaded.

See [Troubleshooting](/troubleshooting) for WebView, Linux shortcut, model,
unlock, updater, and build failures.
