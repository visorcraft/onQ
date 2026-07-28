---
title: Vault security and recovery
description: Authentication modes, encryption layers, recovery procedures, prompt locks, auto-lock, audit logs, and threat boundaries.
---

# Vault security and recovery

onQ uses separate controls for the search-index session, individual prompt
bodies, backups, and the operating-system keychain. They solve different
problems.

## Security layers

| Layer | Protects | Key source |
|---|---|---|
| Vault authentication | The local MongrelDB search index and per-vault state | Master password or generated keychain secret |
| Prompt lock | One selected body in `.onq/locked/` | Fresh per-prompt key in OS keychain |
| Backup seal | Entire `.onqbak` payload | Optional independent archive password |
| Full-disk encryption | Files, app profile, caches, swap, and local backups | Operating system |

Vault authentication does not automatically encrypt ordinary Markdown prompt
bodies. Unlocked prompts are portable plaintext files. Use prompt locks for
selected bodies and full-disk encryption for broad device-at-rest protection.

## Authentication modes

Authentication mode is chosen when a vault is created and recorded as
`password` or `keychain` in `.onq/auth-mode`.

### Master-password vault

The password:

- is supplied by the user;
- is never written by onQ;
- is combined with a random per-vault 32-byte salt;
- is processed with Argon2id;
- must be entered when the vault is opened in a new session.

Current Argon2id parameters:

| Parameter | Value |
|---|---|
| Memory | 64 MiB |
| Iterations | 3 |
| Parallelism | 4 |
| Output | 32 bytes |

The derived key is encoded and passed to the database-open path.

::: danger
There is no password recovery or reset flow. Losing the master password means
losing access to the protected search index and its per-vault state. Portable
unlocked Markdown files may still be readable directly, but that is not a
substitute for a current backup.
:::

### Keychain vault

The no-password path:

1. generates a 24-word English BIP39 phrase;
2. derives a stable secret from that phrase;
3. stores the secret under a per-vault key in the OS keychain;
4. displays the phrase once and requires an explicit saved acknowledgment.

On normal launch, onQ asks the OS keychain for the secret and opens the vault
without an onQ password prompt. The operating system may still require user
login, keyring unlock, biometrics, or another platform approval.

The keychain service name is `onQ`. Per-vault entries are keyed from the
vault's salt so several vaults can coexist.

## Recovery phrase

The 24-word phrase belongs only to a keychain vault. It is not a recovery path
for a password vault and does not recover per-prompt lock keys.

Use it when:

- a keychain entry was deleted;
- the operating-system profile was reset;
- the vault moved to another device;
- onQ reports **Encryption key missing from system keychain**.

Recovery flow:

1. Open the affected vault root.
2. Select **Recover with recovery phrase**.
3. Enter all 24 words in order.
4. onQ validates the BIP39 checksum and word count.
5. onQ derives the secret and verifies it by opening the existing index.
6. Only after successful open does onQ write the secret back to the keychain.

Store the phrase offline. Anyone with the phrase and vault files can derive
the vault secret.

## Prompt locks

Prompt locks protect selected bodies independently from vault authentication.

### Lock flow

1. A fresh random 32-byte data-encryption key is generated.
2. The current body is encrypted with AES-256-GCM.
3. The envelope is written to `.onq/locked/<prompt-id>.enc`.
4. The key is base64-encoded and stored in the OS keychain as
   `prompt:<prompt-id>`.
5. The Markdown body is cleared.
6. Frontmatter is updated to `locked: true`.
7. The search row removes plaintext body and character count while preserving
   the existing semantic embedding.

Metadata remains readable: ID, title, project, tags, favorite state, lock
state, and timestamps.

### Unlock flow

1. Read the per-prompt key from the OS keychain.
2. Read and authenticate the AES-GCM envelope.
3. Restore UTF-8 body text to the Markdown file.
4. Set `locked: false`.
5. Remove the envelope.
6. Delete the per-prompt keychain entry.

If the keychain entry is missing, onQ cannot decrypt that envelope. The vault
password and recovery phrase do not derive prompt-lock keys.

### Prompt-lock limits

Prompt locking does not scrub:

- metadata in the Markdown file;
- a semantic embedding already stored in the protected index;
- previous plaintext snapshots under `.onq/history/`;
- clipboard history from earlier copies;
- backups or external sync copies made before locking.

Delete sensitive history snapshots and old backups manually if your retention
policy requires it. Test recovery before deleting any key material.

## Auto-lock and manual lock

**Lock vault** and idle auto-lock clear the open Vault and database handles
from process state. The UI returns to unlock.

Auto-lock policies:

- Lock on quit only, default;
- Idle timeout from 1 to 1440 minutes;
- Disabled.

Activity includes pointer input, keyboard input, and window focus. The policy
is evaluated every 15 seconds.

Session lock does not rewrite unlocked Markdown bodies or clear the system
clipboard. It protects the open index session, not every local copy of prompt
text.

## Security audit log

Audit logging defaults to enabled and writes local JSON Lines to:

```text
.onq/audit.log
```

An entry contains:

```json
{
  "at": "2026-07-27T12:34:56Z",
  "kind": "vault_lock",
  "detail": "idle_auto_lock"
}
```

Depending on the event, `detail` may expose a local path or prompt ID. The log
never leaves the device through onQ, but it is included in full vault backups
and any external sync of `.onq/`.

Disable logging from **Settings > Vault** when local audit metadata is not
wanted.

## Backup security

A full `.onqbak` archive contains the vault tree:

- unlocked plaintext Markdown bodies;
- history snapshots;
- prompt-lock envelopes;
- protected search index;
- audit log;
- installed plugins.

Use the optional archive password when the file may leave trusted encrypted
storage. The archive password is not the vault password.

Keychain entries are outside the vault and are not included. A restored
keychain vault needs its 24-word phrase when the original keychain entry is
unavailable. Locked prompts need their separate per-prompt keychain entries.

Read [Backups and portability](/backups) for the full restore model.

## Threat boundaries

| Scenario | onQ control | Remaining action |
|---|---|---|
| Someone copies only `.onq/search-index` | Vault authentication gates normal open | Use a strong password or protected OS account |
| Someone reads an unlocked `.md` file | None; body is intentionally portable plaintext | Use prompt lock or full-disk encryption |
| Someone reads a locked `.md` file | Body is absent; envelope is separate | Protect keychain and old history/backups |
| Malicious process runs as the same user | Can often access files, clipboard, input, and process memory | Rely on OS isolation and endpoint security |
| Device is stolen while powered off | Depends on file state and OS storage | Enable full-disk encryption |
| Clipboard manager records a copied prompt | Outside onQ | Configure or clear clipboard history |
| Untrusted native plugin is enabled | Plugin shares onQ process privileges | Do not install it |
| Sync provider account is compromised | Synced vault copies are exposed according to their file state | Use provider encryption and sealed backups |

## Recommended baseline

1. Enable full-disk encryption.
2. Use a strong unique vault password, or protect the OS keychain account.
3. Store the 24-word phrase offline when using keychain mode.
4. Keep audit logging only when its local trail is useful.
5. Use prompt locks selectively and account for history copies.
6. Password-seal backups that leave the device.
7. Test a restore before relying on a backup routine.
8. Install no untrusted native plugins.

## Report a vulnerability

Do not open a public issue. Use
[GitHub private vulnerability reporting](https://github.com/visorcraft/onQ/security/advisories/new)
and include version, platform, reproduction steps, impact, and mitigation.
