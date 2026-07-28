---
title: Feature status
description: Shipped behavior, partial integrations, known limits, and deferred product paths.
---

# Feature status

This page distinguishes end-to-end user behavior from modules or UI that
exist only as scaffolding. It describes the current 1.x repository line.

## Shipped end to end

| Area | Current behavior |
|---|---|
| Vault creation | Fresh directory setup with password or keychain mode |
| Vault reopen | Last/recent vaults, password unlock, keychain auto-open, BIP39 recovery |
| Prompt storage | YAML-frontmatter Markdown under `prompts/` |
| Editing | Drafts, save, delete, project, tags, favorite, body copy |
| Keyword search | Sparse local search over title, tags, project, and body |
| Semantic search | Explicit MiniLM download, local 384-d embedding, ANN and RRF |
| Similarity | MinHash **More like this** |
| Organization | Projects, tags, favorites, recent, unfiled, smart folders |
| Templates | <code v-pre>{{name}}</code> and <code v-pre>{{name&#124;default}}</code> on Palette copy |
| History | Body snapshots, retention, list, and restore |
| Prompt locks | AES-GCM envelope plus per-prompt OS keychain key |
| Session lock | Manual and idle policies |
| Backups | Full versioned `.onqbak`, optional archive seal, destructive restore |
| Markdown transfer | Directory import and full Markdown export |
| Audit | Local JSONL log with Settings toggle and viewer |
| Desktop shell | Tray hide/show, single instance, global shortcut |
| Updates | Stable-feed check on launch and manual version check |
| Localization | English, Thai, Spanish, Chinese, French, German |
| Legal | Bundled licenses, credits, acknowledgments, and runtime notices |

## Partial or scaffolded

### Live filesystem synchronization

Core modules exist for:

- recursive filesystem events;
- per-prompt sync sidecars;
- three-way merge;
- reconciliation;
- conflict parsing and resolution UI.

The desktop shell does not start the vault watcher or route reconcile outcomes
into the conflict UI. External edits are not a supported live-update contract.
Quit onQ before bulk external changes.

### JSON and ChatGPT imports

Core importers support JSON arrays and `conversations.json`. The Settings
picker requests a directory and the shipped flow therefore exposes Markdown
directory import. ZIP extraction is not implemented.

### Plugin runtime

The install, signature, license, enable/disable, registry, native-library load,
and `onq_plugin_init` paths exist.

Current host runtime limits:

- `register_command` records Palette commands;
- registered native handler pointers are not invoked;
- database, vault, keychain, embedding, and config callbacks return `-1`;
- logging callback is a no-op;
- capability-only commands return a status string rather than plugin work.

Treat plugins as an ABI and packaging preview, not a complete extension API.

### Alternate plugin embedders

Enabled plugins with embedding capabilities appear in Settings. Selecting one
changes only the in-process preference string. It does not replace the
built-in MiniLM object or persist a working model choice.

### Beta updates

The beta toggle is persisted, but all checks still use the stable updater
endpoints. No beta feed routing exists.

### Update installation

The updater checks and reports availability. The current UI does not download,
install, or restart into an update.

### Markdown preview

The editor offers a Preview tab, but it displays body lines as text rather
than rendering full Markdown syntax.

## Security and portability limits

### Plaintext source by design

Unlocked prompt bodies are Markdown plaintext. Vault session lock protects
the open index session, not those source files.

### Prompt-lock history

Locking clears the current Markdown body, but prior body snapshots can remain
under `.onq/history/`.

### Prompt-lock key portability

Full backups include lock envelopes but not their OS keychain keys. The vault
recovery phrase does not recover per-prompt keys.

### Unsealed backups

An unsealed `.onqbak` contains plaintext unlocked prompts and history inside a
compressed container.

## Deliberate product boundaries

- No hosted account.
- No telemetry.
- No cloud prompt service.
- No subscription system.
- No built-in sync provider.
- No proprietary plugin license.
- No automatic download of embedding models without user action.
- No trash or delete undo.

## Before depending on a boundary

Use shipped paths for production data. For partial paths, inspect the linked
source and tests, then plan for API and behavior changes. Open an issue or
contribute the missing integration instead of assuming scaffolded code is a
public contract.

Architectural decisions live under
[`docs/adr/`](https://github.com/visorcraft/onQ/tree/master/docs/adr).
