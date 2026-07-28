---
layout: home
title: onQ
hero:
  name: onQ
  text: Your prompts. Your files. Ready on cue.
  tagline: A local-first desktop prompt vault with portable Markdown, a protected local index, and fast keyboard search.
  actions:
    - theme: brand
      text: Install onQ
      link: /installation
    - theme: alt
      text: Quick start
      link: /quick-start
features:
  - title: Search first
    details: Open the palette from anywhere with Win+Q, Meta+Q, or ⌘+Q. Keyword search works immediately; local semantic search is optional.
    link: /search
  - title: Portable by design
    details: Prompt bodies and metadata live in ordinary Markdown files with YAML frontmatter. The search index is derived and disposable.
    link: /file-format
  - title: Local control
    details: No cloud account and no telemetry. Choose a master password or a generated key held by the operating-system keychain.
    link: /security
  - title: Organized without lock-in
    details: Use projects, tags, favorites, recent prompts, smart folders, templates, and per-prompt history.
    link: /library
  - title: Recoverable backups
    details: Export the full vault as a versioned .onqbak archive, with an optional archive password independent of vault authentication.
    link: /backups
  - title: Open source
    details: GPL-3.0-only. Built with Rust, Tauri 2, Svelte 5, and MongrelDB. Development and architecture documentation are public.
    link: /development
---

## What is onQ?

onQ is a desktop prompt manager built around a global command palette. Press
the configured shortcut, type a few words, and copy a prompt without moving
your library into a hosted service.

Each prompt is a Markdown file under the vault's `prompts/` directory. onQ
stores search state, history, backups metadata, smart folders, and plugin state
under the vault's `.onq/` directory. See the [vault layout](/vault-layout) for
the exact paths and the [security guide](/security) for what is and is not
encrypted.

## Design promises

- **Local first:** no account, hosted database, subscription, or telemetry.
- **Portable source files:** the prompt library remains useful without onQ.
- **Optional network use:** keyword search is offline; the local embedding
  model downloads only after you ask for it. The app also checks its GitHub
  release feed for update metadata.
- **Explicit recovery:** keychain vaults show one 24-word recovery phrase.
  Password vaults have no recovery mechanism.
- **Honest status:** shipped behavior and scaffolded future paths are separated
  in the [feature status](/feature-status).

## Choose a path

- New user: [install onQ](/installation), then follow the
  [quick start](/quick-start).
- Existing user: browse the [feature guide](/usage), [security model](/security),
  and [backup guide](/backups).
- Contributor: start with [build and test](/development), then read the
  [architecture](/architecture).
- Plugin author: read the [plugin guide](/plugins) and its current runtime
  limitations before building.
