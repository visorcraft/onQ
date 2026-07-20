---
title: Installation
---

# Installation

onQ is a desktop application built with [Tauri 2](https://tauri.app). The recommended
path for most users is to download a pre-built installer from the
[releases page](https://github.com/visorcraft/onQ/releases).

## System requirements

| Platform | Minimum version | Notes                                       |
|----------|-----------------|---------------------------------------------|
| Windows  | Windows 10 64-bit | WebView2 runtime required (preinstalled on Win11) |
| macOS    | macOS 11 Big Sur | Apple Silicon and Intel supported           |
| Linux    | glibc 2.31+     | Tested on Ubuntu 22.04, Fedora 39, Arch     |

## Install a pre-built release

1. Go to [github.com/visorcraft/onQ/releases](https://github.com/visorcraft/onQ/releases).
2. Download the installer for your platform:
   - **Windows:** `onQ_<version>_x64-setup.exe` (NSIS) or `.msi`
   - **macOS:** `onQ_<version>_universal.dmg`
   - **Linux:** `onQ_<version>_amd64.AppImage` or `.deb`
3. Run the installer. On first launch, follow the 4-step tutorial to create your vault and
   generate (or supply) a master passphrase.

### Auto-update

Once installed, onQ checks for new releases on launch. To opt into beta releases,
open **Settings → Updates → Beta channel**.

## Building from source

Building from source requires Node 24+, Rust stable, and platform-specific dependencies
(see the [Tauri prerequisites](https://tauri.app/start/prerequisites/)).

### 1. Clone the repo

```bash
git clone https://github.com/visorcraft/onQ
cd onQ
```

The encrypted search index depends on
[`mongreldb-core`](https://crates.io/crates/mongreldb-core) from crates.io
(declared in the workspace `Cargo.toml`).

### 2. Install Node and Rust

```bash
# Node 24 via nvm (or use your distro's package manager)
nvm install 24 && nvm use 24

# Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 3. Install dependencies

```bash
npm install
```

### 4. Run in development mode

```bash
npm run dev:app
```

This starts Vite for the Svelte frontend and launches the Tauri shell pointing at the dev server.
The first launch downloads the embedding model (`all-MiniLM-L6-v2`) into the user's cache directory.

### 5. Build a production bundle

```bash
npm run build:app
```

The produced installer lands in `target/release/bundle/` (or
`target/<triple>/release/bundle/` when building with an explicit target).

## Verifying your install

After launching, the command palette opens with **Ctrl+L / ⌘K**. Type any
phrase to search the bundled tutorial prompts. If you see ranked results with snippet
highlights, your search index and embeddings are working correctly.

## Troubleshooting

- **"WebView2 not found" on Windows 10:** install the [Evergreen runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).
- **Model download fails behind a corporate proxy:** set `HTTPS_PROXY` in your shell before
  launching, or pre-place the ONNX file in `~/.cache/onQ/models/`.
- **`vault/` is empty after install:** the tutorial vault is created on first run; if you
  skipped the tutorial, run it again from **Help → Replay tutorial**.
