---
title: Architecture
description: System boundaries, repository layers, runtime flows, storage model, search pipeline, security paths, testing, and extension seams.
---

# Architecture

onQ separates portable source files, a derived search database, desktop IPC,
and the Svelte presentation layer.

## System view

```text
User
  |
  v
Svelte 5 UI
  |
  | src/lib/api/* wrappers
  v
Tauri 2 commands in onq-app
  |
  +-------------------+-------------------+------------------+
  |                   |                   |                  |
  v                   v                   v                  v
onq-core           MongrelDB          OS keychain       OS services
  |                   |                   |             tray/dialog/
  v                   v                   v             shortcut/updater
Markdown vault    .onq/search-index   vault + prompt
and .onq files                         lock keys
```

The UI never reaches filesystem, database, or keychain code directly.

## Layer responsibilities

### `src/`

The Svelte frontend owns:

- navigation and modal state;
- Palette interaction;
- Library browsing and organization;
- Settings and About surfaces;
- clipboard actions;
- Tauri command wrappers;
- localization;
- presentation and accessibility.

Every IPC domain has a wrapper under `src/lib/api/`. Components consume those
wrappers, not raw `invoke` calls.

### `crates/onq-app`

The Tauri shell owns:

- command registration and wire DTOs;
- open process state;
- OS dialogs;
- tray and window lifecycle;
- global shortcut backends;
- updater checks;
- real keychain activation;
- legal document embedding;
- bridging async IPC to synchronous core/database work.

`AppState` holds the current Vault, database handle, optional MiniLM model,
auto-lock state, plugin commands, embedder preference, and audit toggle.

### `crates/onq-core`

Core owns platform-light domain behavior:

- Markdown vault and frontmatter;
- database schema and migrations;
- KDF, recovery, keychain trait, and body crypto;
- sparse, vector, and MinHash search helpers;
- history and backups;
- project-path rules;
- smart-folder parser and visual codec;
- import/export;
- plugin verification/loading;
- sync watcher, sidecars, merge, and reconcile primitives.

Core functions use `CoreResult` and typed errors.

### `crates/onq-plugin-sdk`

Defines ABI version 1, the `HostApi` C-compatible function table, and the
`export_plugin!` macro. The declared ABI is broader than the currently
implemented host callbacks. See [Plugin authoring](/plugins).

### `crates/onq-test-utils`

Shared deterministic fixtures and a mock embedding path keep normal tests
offline.

## Vault creation

```text
pick directory
  -> refuse existing .onq/search-index
  -> choose password or keychain mode
  -> create/read random .onq/salt
  -> Argon2id(secret, salt)
  -> create encrypted MongrelDB
  -> run schema migrations
  -> store auth-mode
  -> keychain mode: store derived phrase secret
  -> remember vault in app config
```

`Vault::new` ensures `prompts/` and `.onq/` exist. Database migrations create
six tables and seed per-vault application state.

## Vault open

```text
remembered or selected path
  -> require .onq/search-index + .onq/salt
  -> read auth-mode
  -> password: request user secret
  -> keychain: read per-vault secret
  -> derive database passphrase
  -> Db::open
  -> migrate
  -> reconcile embedding index preference
  -> backfill sparse representations from prompt files
  -> expose open state to UI
```

Keychain recovery verifies the phrase by opening the database before it
updates the OS keychain.

## Prompt write

```text
Editor save
  -> create ULID when draft
  -> normalize metadata
  -> snapshot body history
  -> render YAML + Markdown
  -> atomic temporary write + rename
  -> prune history
  -> build search cells
       title/tags/project/body sparse vector
       character-trigram MinHash set
       preserve or compute 384-d embedding
  -> upsert MongrelDB row
```

The Markdown file is canonical user content. The database row is a derived
search representation plus per-vault application state.

## Search

Palette typing is debounced by 150 ms. The Tauri command moves synchronous
database work into `spawn_blocking`.

```text
query text
  -> optional MiniLM query embedding
  -> hard metadata filters, when called by smart folder
  -> sparse retriever
  -> optional ANN retriever
  -> Reciprocal Rank Fusion
  -> recency and favorite tie-breakers
  -> metadata + safe snippet DTO
```

Keyword search does not depend on the model. MiniLM download is explicit and
stores model files in the app cache. Binary or dense ANN preference is stored
per vault.

**More like this** uses a separate MinHash path over character trigrams.

## Prompt lock

```text
save current prompt
  -> random per-prompt 256-bit key
  -> AES-256-GCM body envelope
  -> .onq/locked/<id>.enc
  -> base64 key in OS keychain
  -> clear Markdown body
  -> mark locked
  -> remove sparse body data, preserve embedding
```

Unlock reverses the path and deletes both envelope and keychain key after body
restoration.

## Backup

Full export:

```text
walk vault tree
  -> skip .onq-pre-import-* leftovers
  -> gzip tar payload
  -> optional Argon2id + AES-GCM outer seal
  -> versioned .onqbak
```

Full import validates and extracts into staging, releases the open database
handle, moves live contents aside, promotes staging, and restores previous
contents on a failed promotion where possible.

## Storage schema

The prompts table uses stable column IDs for:

- identity and metadata;
- body;
- character count and timestamps;
- 384-dimensional embedding;
- encoded sparse vector;
- encoded MinHash members.

Indexes include bitmap metadata indexes, learned-range indexes, full-text,
ANN, sparse, and MinHash kinds.

`app_state` is a singleton row for schema version and per-vault preferences.
Migrations are ordered Rust closures. Versions are never reordered, removed,
or reused.

## Blocking work

MongrelDB is synchronous. Tauri commands use `tokio::task::spawn_blocking`
for database, model, keychain, archive, and filesystem work that would
otherwise stall the async runtime.

Frontend search failures preserve previous results rather than crashing the
Palette.

## Desktop lifecycle

- Single-instance plugin focuses the first process on a second launch.
- Closing and minimizing hide the main window.
- Tray left-click and **Show onQ** restore it.
- Tray **Quit** exits.
- Windows/macOS use Tauri global shortcuts.
- Linux reads keyboard input event devices.
- Frontend activity updates the idle timer.
- Stable updater metadata is checked on launch.

## Local-first boundary

Prompt contents and inference remain local. Expected network paths are:

- GitHub updater metadata;
- explicit Hugging Face model download;
- browser-opened external links.

External storage providers operate on vault files independently from onQ.
No hosted sync or account service exists.

## Extension boundary

Plugin installation verifies:

- gzip-tar extraction safety;
- exact GPL-3.0-only license;
- raw ed25519 signature over `manifest.toml`;
- compiled trust anchor;
- ABI version on library load.

The runtime currently loads libraries and calls initialization, but most
HostApi callbacks and handler execution remain scaffolded.

## Sync boundary

`notify`, sidecar, base snapshot, reconcile, diff merge, and conflict
components are implemented and tested independently. They are not connected
to the desktop lifecycle. This is deliberate status, not an invitation to
remove those modules as dead code.

## Testing architecture

| Layer | Primary checks |
|---|---|
| Pure TypeScript and stores | Vitest |
| Svelte components | Vitest + Storybook |
| Core domain logic | Rust unit tests |
| Vault/database compatibility | Rust integration tests |
| Desktop UI accessibility | Playwright + axe |
| Full user paths | Playwright E2E |
| Real embedding | Opt-in heavy integration test |
| Performance | Criterion and regression script |

The smallest relevant test should accompany every behavior change.

## Architecture records

ADRs live under
[`docs/adr/`](https://github.com/visorcraft/onQ/tree/master/docs/adr).
Current plugin embedder direction is recorded in
[`0001-plugin-embedders.md`](https://github.com/visorcraft/onQ/blob/master/docs/adr/0001-plugin-embedders.md).
