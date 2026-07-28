---
title: Privacy and network access
description: What stays local, which network requests can occur, and how plugins, clipboard, sync, and logs affect privacy.
---

# Privacy and network access

onQ requires no cloud account and contains no telemetry pipeline. Prompt
storage, indexing, embedding inference, settings, history, and audit data are
local.

## Local data

| Data | Location |
|---|---|
| Prompt source | `<vault>/prompts/*.md` |
| Search and per-vault state | `<vault>/.onq/search-index/` |
| History | `<vault>/.onq/history/` |
| Prompt-lock envelopes | `<vault>/.onq/locked/` |
| Audit log | `<vault>/.onq/audit.log` |
| Plugins | `<vault>/.onq/plugins/` |
| Last/recent vaults and auto-lock policy | Tauri application config directory |
| Language, shortcut, and recent prompt IDs | Local WebView profile |
| MiniLM files | Tauri application cache directory |
| Vault and prompt keys | Operating-system keychain |

Use **Settings > Backups** to display and copy the current vault and search
index paths.

## Network paths

### GitHub update metadata

On launch, onQ silently checks configured GitHub release endpoints for Tauri
updater metadata. Manual checks are available in Settings and through the Home
version button.

The request checks application version metadata. onQ does not attach prompt
contents. Startup failures stay silent. There is no current UI switch to
disable this check.

### Hugging Face model download

The built-in semantic model downloads only after selecting **Load MiniLM
model**:

```text
sentence-transformers/all-MiniLM-L6-v2
```

The download fetches `onnx/model.onnx` and `tokenizer.json` from Hugging Face
Hub into the app cache. After that:

- prompt embedding runs locally;
- query embedding runs locally;
- prompt text is not sent to Hugging Face;
- keyword search remains available without the model.

### External links

Opening repository, release, or documentation links delegates to the platform
browser. The destination site then applies its own privacy policy.

## What onQ does not provide

- no user account;
- no hosted prompt database;
- no telemetry or usage analytics;
- no advertising SDK;
- no remote prompt inference;
- no cloud synchronization service;
- no automatic upload of audit logs or crash reports.

## Clipboard

Copying is a core workflow. Clipboard content is visible to the operating
system and may be retained by:

- clipboard history;
- desktop-environment clipboard managers;
- remote-desktop software;
- other local applications.

onQ does not automatically clear copied prompt text. Configure the operating
system when clipboard retention is a concern.

## Keychain

Keychain-mode vault secrets and per-prompt lock keys use the platform keyring
through the Rust `keyring` library. Access behavior depends on the operating
system, desktop environment, user session, and keyring configuration.

Linux systems need a working secret service. A missing or locked secret
service can prevent keychain vault or prompt-lock access.

## Audit and application logs

The optional audit log is local but can contain:

- vault paths;
- archive paths;
- prompt IDs;
- event timestamps.

Runtime tracing may also contain diagnostic paths and errors. Share logs only
after reviewing them for local identifiers.

## Native plugins

Plugins are signed native libraries loaded into the onQ process. Signature and
license checks verify package policy, not harmless behavior.

An enabled native plugin has process-level access and can potentially bypass
application privacy expectations. The current SDK callback surface is
incomplete, but normal native-code privileges still apply. Install only
packages you trust.

## External synchronization

If a vault lives inside Git, Dropbox, iCloud, Syncthing, or another provider,
that provider receives whichever vault files its configuration includes.
onQ does not control:

- provider-side encryption;
- retention and version history;
- collaborator access;
- conflict behavior;
- metadata collection.

Unlocked Markdown files are plaintext. Prompt-lock keys remain in the local
OS keychain, so syncing only the envelope does not make a locked prompt
portable.

## Delete and retention

Deleting a prompt removes its current file and index row. Copies may remain
in:

- `.onq/history/`;
- full or Markdown backups;
- sync-provider history;
- Git history;
- clipboard history;
- filesystem snapshots.

Data erasure requires clearing every applicable layer.
