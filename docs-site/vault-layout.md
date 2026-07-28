---
title: Vault layout
description: Exact vault tree, derived index state, app-profile state, model cache, and which paths are portable or disposable.
---

# Vault layout

A vault is a user-selected directory with portable prompt source and onQ
metadata.

## Tree

```text
<vault>/
├── prompts/
│   └── <prompt-id>.md
└── .onq/
    ├── auth-mode
    ├── salt
    ├── search-index/
    ├── history/
    │   └── <prompt-id>/
    │       └── <RFC3339 timestamp>.md
    ├── locked/
    │   └── <prompt-id>.enc
    ├── audit.log
    ├── plugins/
    │   └── installed/
    │       └── <plugin-id>/
    ├── state/
    │   └── <prompt-id>.json
    └── bases/
        └── <sha256>.txt
```

Only `prompts/`, `auth-mode`, `salt`, and `search-index/` are guaranteed in a
fresh or initialized vault. Other paths are created lazily by their feature.
`state/` and `bases/` belong to sync/reconcile scaffolding and may remain
absent in the current desktop path.

## Portable prompt source

`prompts/<id>.md` is the canonical source for prompt metadata and unlocked
body text. Files are:

- UTF-8;
- YAML frontmatter plus Markdown body;
- named by prompt ID;
- suitable for normal file backup and inspection.

See [Prompt file format](/file-format).

## `.onq/auth-mode`

Plain text with one value:

```text
password
```

or:

```text
keychain
```

Do not change it manually. Authentication secrets and recovery behavior
depend on the selected mode.

## `.onq/salt`

A raw 32-byte per-vault salt used by the Argon2id key derivation path. It is
not a password, but it is required to derive the same database key.

Include it in every full backup.

## `.onq/search-index/`

The MongrelDB database contains six logical tables:

| Table | Role |
|---|---|
| `prompts` | Searchable prompt metadata, body state, sparse data, MinHash, and embeddings |
| `app_state` | Schema version and per-vault preferences |
| `folders` | Registered logical project paths |
| `smart_folders` | Saved names, DSL, and visual representation |
| `embedding_queue` | Deferred embedding work state |
| `plugins` | Installed plugin registry and enabled state |

Prompt source remains in Markdown. The index is derived and can be rebuilt,
but the current application does not expose a one-click delete-and-rebuild
command. Do not edit MongrelDB files directly.

The index schema uses stable column IDs and versioned migrations. Current
migrations cover initial tables, minimize-on-copy state, and search/history/
backup preferences.

## `.onq/history/`

Body-only snapshots, grouped by prompt ID and named with RFC 3339 timestamps.
They are included in full backups.

History may retain sensitive plaintext after a prompt is locked. Retention
pruning occurs during writes.

## `.onq/locked/`

AES-256-GCM prompt-body envelopes. The corresponding per-prompt key is in the
OS keychain, not in this directory.

Copying this directory to another device does not copy its keys.

## `.onq/audit.log`

Append-only JSON Lines security events. The newest entries are displayed
first in Settings. The file can contain local paths and prompt IDs.

## `.onq/plugins/`

Verified plugin archives are extracted under `installed/<plugin-id>/`.
Native libraries execute with the application process's privileges.

## Sync scaffolding

The core sync design uses:

- `.onq/state/<id>.json` for base/ours/theirs hashes, sync time, edit count,
  and tombstone state;
- `.onq/bases/<sha256>.txt` for content-addressed merge bases.

Watcher and reconcile modules exist, but the shipped app shell does not yet
run this full workflow. See [Feature status](/feature-status).

## State outside the vault

Some data must be available before unlock.

### Tauri application config

The platform application-config directory can contain:

```text
last-vault
recent-vaults.json
auto-lock-policy
audit-enabled
```

Recent vaults are capped at ten.

### WebView profile

Local storage can contain:

```text
onQ.globalShortcut
onq.locale
onq.recent_prompt_ids
```

Recent Palette prompt IDs are capped at five.

### Application cache

Optional MiniLM files live under the Tauri application cache:

```text
all-MiniLM-L6-v2/
├── model.onnx
└── tokenizer.json
```

Deleting this cache removes semantic model availability, not prompt files.
Keyword search continues.

### Operating-system keychain

The keychain can contain:

- one derived secret per keychain vault;
- one data-encryption key per currently locked prompt.

The full backup archive cannot copy these entries.

## What can be deleted?

| Path | Safe consequence |
|---|---|
| App model cache | Semantic search unavailable until model reload/download |
| `.onq/search-index/` | Derived state lost; manual reconstruction support is limited |
| `.onq/history/` | Restore history permanently lost |
| `.onq/audit.log` | Audit history permanently lost |
| `.onq/locked/` | Locked bodies permanently lost unless another envelope copy exists |
| `.onq/salt` | Existing index key derivation breaks |
| `prompts/` | Canonical prompt source lost |

Prefer a full backup before deleting any vault path.
