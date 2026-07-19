---
layout: home
title: onQ
hero:
  name: onQ
  text: Search-first prompt vault
  tagline: Local-first. Hybrid Markdown + MongrelDB. Soft glass aesthetic.
  actions:
    - theme: brand
      text: Get Started
      link: /installation
    - theme: alt
      text: View on GitHub
      link: https://github.com/visorcraft/onQ
features:
  - title: Spotlight-centric UI
    details: Ctrl+L / ⌘K is the app. Search prompts, folders, tags, smart folders, commands, and plugins from a single command palette.
  - title: Hybrid search
    details: Keyword (BM25/FTS) + semantic (vector cosine) fused via Reciprocal Rank Fusion. Six MongrelDB index kinds, recency decay.
  - title: Markdown vault
    details: Your prompts live in plain .md files with YAML frontmatter. Sync with Git, Dropbox, iCloud, or Syncthing.
  - title: Two-layer encryption
    details: Master passphrase plus per-prompt keys via the OS keychain (macOS Keychain, Windows DPAPI, Linux libsecret).
  - title: Plugin system
    details: Rust native plugins via C ABI, ed25519-signed manifests, GPL-3.0-only enforced. Extend without forking.
  - title: Cross-platform
    details: Windows, macOS, Linux. Built with Tauri 2. Auto-update via GitHub releases.
---

## What is onQ?

onQ is a desktop prompt manager for AI power users. Search is the heart:
hybrid BM25 + vector cosine over six MongrelDB index kinds, fused with recency decay.

Your prompts live in plain `.md` files (sync with Git, Dropbox, iCloud, Syncthing — your choice)
and the search index in encrypted local storage. Local-first means your data stays on your device;
no remote calls by default.

## Why onQ?

Most prompt managers treat prompts as opaque blobs behind a search box. onQ treats them
as **plain text you own**, with **structured metadata** you can edit in any text editor, indexed
with a **production-grade hybrid search engine** that combines lexical and semantic relevance.

- **No vendor lock-in.** Export the whole vault as a folder of `.md` files at any time.
- **No telemetry.** Zero remote calls unless you opt in.
- **No subscriptions.** GPL-3.0-only, free forever.
- **WCAG 2.1 AA.** Keyboard-first, screen-reader tested, reduced-motion honored.

## Where to next?

- New to onQ? Start with **[Installation](/installation)** to get the app running.
- Want to understand what you can do with it? See **[Usage](/usage)** for the feature tour.
- Want to build it yourself? See **[Building from source](/installation#building-from-source)**.
