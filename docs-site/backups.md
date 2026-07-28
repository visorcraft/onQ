---
title: Backups and portability
description: Full vault backups, Markdown import/export, restore behavior, key portability, and safe external synchronization.
---

# Backups and portability

onQ has two different export paths:

| Path | Output | Purpose |
|---|---|---|
| Prompt export | Directory of `.md` files | Portability and bulk text exchange |
| Vault backup | One `.onqbak` file | Complete onQ state and disaster recovery |

Use both when possible. Markdown export is easy to inspect. A full backup
preserves the state needed to restore onQ.

## Full `.onqbak` backup

Open **Settings > Backups**. The page shows the current vault root and
`.onq/search-index` paths.

### What it contains

The exporter walks the whole vault tree, including:

- `prompts/`;
- `.onq/search-index/`;
- `.onq/salt` and `.onq/auth-mode`;
- `.onq/history/`;
- `.onq/locked/`;
- `.onq/audit.log`;
- `.onq/plugins/`;
- any other ordinary file stored inside the selected vault root.

Temporary `.onq-pre-import-*` recovery directories are skipped.

### Container format

Version 1 starts with:

```text
magic[8] = ONQBAK + major 1 + minor 0
flags[1] = 0 plain | 1 sealed
```

The payload is a gzip-compressed tar tree.

With archive password protection:

1. onQ generates a fresh 32-byte salt;
2. derives a 32-byte key with Argon2id;
3. seals the entire compressed payload with AES-256-GCM.

The archive password is independent from the vault's master password or
keychain secret.

### Plain archive warning

An unsealed archive is only a container, not a confidentiality boundary.
Unlocked prompt bodies and history snapshots remain plaintext inside it.
Use archive protection or place the file on trusted encrypted storage.

### Export

1. Open **Settings > Backups**.
2. Optionally enable **Password-protect archive**.
3. Enter and confirm an archive password.
4. Select **Export backup**.
5. Save the `.onqbak` outside the vault.

Successful export updates the vault's backup timestamp. The default reminder
becomes overdue after seven days.

## Restore a full backup

Full import replaces the currently open vault at its current path.

1. Open the target vault.
2. Open **Settings > Backups**.
3. Select **Import backup** and choose a `.onqbak`.
4. Enter the archive password when the archive is sealed.
5. Review the destructive replacement confirmation.
6. Confirm.

The backend:

1. closes the database handle;
2. validates the archive header and password;
3. extracts into a temporary staging directory;
4. verifies that the payload contains `.onq/salt` and
   `.onq/search-index/`;
5. moves current target contents into a temporary recovery directory;
6. copies staged contents into the target;
7. restores the previous contents when promotion fails where possible;
8. removes temporary recovery data after success;
9. leaves the vault session locked.

You must unlock the restored vault again.

::: danger
A successful import removes every previous file under the target vault root,
not only onQ-managed files. Do not place unrelated files inside a vault that
you may restore in place.
:::

## Authentication after restore

### Password vault

Use the same master password that protected the vault when the backup was
created. The backup's optional outer password only opens the archive.

### Keychain vault

OS keychain entries are not inside `.onqbak`.

- On the original profile, the existing per-vault keychain entry may still
  work because the restored salt matches.
- On another profile or device, recover with the original 24-word phrase.

### Locked prompts

Per-prompt keys are also stored outside the vault. A backup contains encrypted
`.enc` envelopes but not those keys. Moving a locked prompt to a device
without its keychain entry makes it unreadable.

Unlock sensitive prompts before a migration when portability matters, then
protect the resulting backup with an archive password.

## Markdown export

Open **Settings > Vault > Import / export prompts** and select **Export**.
The UI writes all readable prompts to a selected directory as:

```text
<prompt-id>.md
```

Files keep full supported YAML frontmatter and body. Locked prompt files have
an empty body because the ciphertext envelope is not part of a Markdown
export.

The underlying core supports filters by any tag, exact project, favorites,
or explicit prompt IDs. The current Settings UI exports all prompts and does
not expose those filters.

## Markdown import

The current desktop picker accepts a directory. Import:

- scans `.md` files recursively;
- stops below four nested directory levels;
- skips non-Markdown files;
- preserves valid onQ frontmatter;
- treats invalid or missing onQ frontmatter as plain Markdown;
- derives a plain file's title from its filename;
- generates default metadata and a new ID for plain files;
- skips an existing prompt with the same ID;
- reports created, skipped, and error counts;
- reindexes the resulting vault best-effort.

Directory hierarchy does not become project hierarchy automatically.

## Other importer code

Core import modules also support:

- a JSON array with optional `id`, `title`, `body`, `tags`, and `folder`;
- a ChatGPT `conversations.json` file or containing directory.

The shipped Settings file picker requests directories and always uses auto
detection. It therefore exposes Markdown-directory import, not arbitrary JSON
file selection. A filename ending in `.zip` is detected as ChatGPT data by the
core, but ZIP extraction is not implemented. Extract the export and use a
supported path through tooling or wait for a dedicated UI.

## External sync

Portable Markdown makes Git, Dropbox, iCloud, Syncthing, and similar tools
possible storage choices. onQ does not provide a hosted sync service.

The repository contains a filesystem watcher, sidecar state, three-way merge
logic, and conflict UI components, but the desktop shell does not yet connect
them end to end.

Safe current practice:

1. Make a full backup.
2. Quit onQ from the tray before bulk external edits or sync conflict
   resolution.
3. Let the external tool finish.
4. Inspect conflicted files manually.
5. Reopen the vault.
6. Verify Library and search results before continuing.

Do not assume an edit made in another process will appear live in an already
open onQ session.

## Backup checklist

- Keep at least one backup outside the vault and outside its sync folder.
- Seal backups that contain sensitive unlocked prompts.
- Save the vault password or recovery phrase separately from the archive.
- Account for per-prompt keychain keys.
- Test restore into a disposable copy before an emergency.
- Keep more than one generation.
- Verify backup creation after major upgrades.
