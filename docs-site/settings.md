---
title: Settings reference
description: Every shipped Settings section, default, scope, and current limitation.
---

# Settings reference

Settings are grouped into General, Search, Vault, Backups, Plugins, and
Updates. Some preferences are per vault; others must work before unlock and
are stored in the application profile.

## General

### Global shortcut

Default:

- Windows: **Win+Q**
- Linux: **Meta+Q**
- macOS: **⌘+Q**

Select the shortcut control, then press Ctrl, Alt, or the platform Meta key
plus another supported key. Escape cancels recording.

Windows and macOS use Tauri's native global shortcut backend. Linux reads
keyboard input devices directly and needs permission to open them. See
[Linux shortcut permission](/troubleshooting#linux-global-shortcut-does-not-work).

The shortcut is application-wide and stored in the local WebView profile.

### Minimize after copy

When enabled, selecting a prompt's main Palette result:

1. copies the body;
2. closes the Palette;
3. hides onQ to the system tray.

The default is off. This setting is stored per vault.

### Theme

Choose Dark or Light. Theme is stored per vault in the protected app-state
row. Before unlock, the first paint uses Dark.

### Language

Available UI languages:

- English
- Thai
- Spanish
- Chinese
- French
- German

The saved locale is application-wide and works before unlock. Without a saved
choice, onQ matches the browser/WebView language and falls back to English.

## Search

### Embedding quantization

- **Binary**, default: binary ANN candidates plus exact cosine reranking.
- **Dense:** full-precision ANN after index rebuild.

Controls remain disabled while search is sparse-only because no embedding
model is loaded. Changing mode rebuilds the derived embedding index.

### Search status

The status panel reports:

- keyword index on;
- model loaded, cached, or not installed;
- model ID;
- sparse-only, binary ANN, dense pending, or dense live;
- refresh action.

**Load MiniLM model** downloads and loads the optional local model, then
re-embeds all prompts. Once loaded, the same button is labeled **Re-embed
vault**.

### Active embedder

Built-in MiniLM is the functional default. Enabled plugins that advertise an
embedding capability can appear in the selection list.

The current preference setter stores only in process memory. It does not yet
replace the built-in `AppState.embedder`. See [Feature status](/feature-status).

### Search recency period

Default: 30 days. Allowed UI range: 1 through 3650.

The value controls how quickly the small recency ranking bonus decays. It does
not remove old prompts or limit results to a date range.

## Vault

### Auto-lock

| Policy | Behavior |
|---|---|
| Lock on quit only, default | No idle timeout; a new process must reopen the vault |
| Idle timeout | Clear the vault session after 1 to 1440 idle minutes |
| Disabled | Never auto-lock because of idle time or policy evaluation |

Pointer, keyboard, and window-focus activity reset the idle timer. The
frontend checks the policy every 15 seconds.

Locking clears the open vault and database handles from memory. It does not
encrypt ordinary unlocked Markdown bodies. The auto-lock preference is
application-wide and stored outside the vault so it can apply before unlock.

Use **Lock vault** in the Palette for an immediate session lock.

### History retention

Default: 30 days. Allowed UI range: 0 through 3650.

The value controls body snapshots under `.onq/history/`. Pruning happens
during prompt writes.

### Import and export prompts

The current desktop picker selects a directory:

- **Import** recursively reads `.md` files up to four directory levels below
  the chosen directory.
- Files with valid onQ frontmatter preserve supported metadata.
- Plain Markdown gets a generated ID, a filename-derived title, and default
  metadata.
- Conflicting IDs are skipped by the UI path.
- **Export** writes every readable prompt as `<id>.md` into a selected
  directory.

The core also contains JSON-array and ChatGPT export importers, but the shipped
Settings picker does not expose file selection for them. See
[Feature status](/feature-status).

### Security audit log

Audit logging defaults to enabled. The Settings panel can disable it and list
the newest 80 events.

Events can include:

- vault unlock and lock;
- backup import and export;
- Markdown import and export;
- prompt unlock;
- history restore;
- plugin install.

The append-only JSON Lines file is `.onq/audit.log`. Details can contain local
paths or prompt IDs. The enabled preference is application-wide.

### Authentication information

Password vaults show their auth mode.

Keychain vaults allow the raw derived vault secret to be displayed only after
the matching 24-word recovery phrase is entered. Treat that value and the
phrase as secrets. Close Settings after use.

## Backups

The Backups section shows copyable absolute paths for:

- the vault root;
- `.onq/search-index`.

It can export or destructively import a full `.onqbak` archive. Archive
password protection is independent of the vault's authentication mode.

The backup reminder defaults to seven days and appears when no recent full
backup export is recorded. There is no reminder interval control in the
current Settings UI.

Read [Backups and portability](/backups) before importing.

## Plugins

The Plugins section can:

- install a `.onqplugin`, `.tar`, `.gz`, or `.tgz` selected by the file
  picker;
- list installed plugins;
- enable or disable a plugin;
- refresh state;
- uninstall a plugin.

The installer itself expects a gzip-compressed tar archive with
`manifest.toml`, `signature.sig`, and a native library. It rejects an
untrusted signature, a non-GPL-3.0-only license, a duplicate ID, and archive
path traversal.

Plugins are native code with the same process privileges as onQ. Install only
trusted packages. Read [Plugin authoring](/plugins) for current runtime limits.

## Updates

onQ silently checks the configured stable GitHub updater feed on launch. A
manual check is available in Settings and from the Home version button.

The current action reports that a version is available or that onQ is current.
It does not download and install the update from this UI. Follow the release
link manually when needed.

### Beta channel

The toggle persists a per-vault beta preference. Current updater configuration
still uses the stable feed regardless of this value. Beta routing has not
shipped.

## Preference scope

| Preference | Scope |
|---|---|
| Global shortcut | Application/WebView profile |
| Language | Application/WebView profile |
| Auto-lock policy | Application config |
| Audit enabled | Application config |
| Theme | Per vault |
| Minimize after copy | Per vault |
| Search recency period | Per vault |
| History retention | Per vault |
| Embedding quantization | Per vault |
| Beta preference | Per vault |
| Backup timestamp/reminder state | Per vault |
