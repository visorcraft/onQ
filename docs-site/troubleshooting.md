---
title: Troubleshooting
description: Resolve install, vault, keychain, shortcut, search, model, backup, update, and source-build failures.
---

# Troubleshooting

Start with the exact error text, onQ version, operating system, vault auth
mode, and the action that produced it. Avoid posting vault files, passwords,
recovery phrases, or keychain values.

## App does not start

### Windows WebView2 error

Install Microsoft's current WebView2 Evergreen Runtime, then relaunch onQ.
Current Windows releases usually include it.

### Linux missing shared library

Install the native packages listed by the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/). The Ubuntu
CI build installs:

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf
```

Package names differ on Fedora, Arch, and other distributions.

### Window disappeared

Closing or minimizing hides the window to the tray. Use the global shortcut,
left-click the tray icon, or select **Show onQ** from its menu. Use the tray
menu's **Quit** action to stop the process.

## Vault creation says it already exists

onQ refuses to create over an existing `.onq/search-index/`. Choose **Open
existing vault** instead, or select a different directory.

Do not delete the index just to bypass this guard. First export a full backup
and verify which vault is at that path.

## “vault not found”

Select the vault root, not `prompts/` or `.onq/search-index/`. A recognized
vault contains both:

```text
.onq/salt
.onq/search-index/
```

If either is missing, restore a full `.onqbak` backup. A directory containing
only exported Markdown is not an existing onQ vault. Create a new vault and
import that directory.

## Master password fails

- Verify that the selected path is the intended vault.
- Check keyboard layout and Caps Lock.
- Use the password from vault creation, not an archive password.
- Do not use a recovery phrase with a password vault.

There is no password-reset path. If the password is lost, restore from a
usable backup or import readable Markdown into a new vault.

## Keychain vault asks for recovery

The per-vault keychain entry is missing or unavailable.

1. Make sure the platform keyring is running and unlocked.
2. Confirm the vault belongs to this OS user profile.
3. Select **Recover with recovery phrase**.
4. Enter the original 24 English BIP39 words in order.

Recovery validates the phrase against the existing index before writing a new
keychain entry.

On Linux, verify that a Secret Service implementation is available. Desktop
environments commonly provide it through GNOME Keyring, KWallet integration,
or another compatible service.

## Locked prompt will not unlock

Common errors:

- `lock key not found in keychain`
- malformed key data
- missing `.onq/locked/<id>.enc`
- authentication failure from a changed or corrupt envelope

The vault password and 24-word vault recovery phrase cannot recreate a
per-prompt key. Restore both the envelope and matching OS keychain entry from
the original environment, or recover the plaintext from history or an older
backup.

Do not overwrite or delete the envelope while investigating.

## Linux global shortcut does not work

The Linux backend reads keyboard event devices. If it reports:

```text
Could not read Linux input devices. Add this user to the input group.
```

Inspect the group:

```bash
getent group input
```

On distributions that use the `input` group, an administrator can add the
current user:

```bash
sudo usermod -aG input "$USER"
```

Log out of the entire desktop session and back in before retrying.

::: warning
Membership in `input` can grant access to all keyboard events, not only onQ's
shortcut. Follow your distribution's security policy. A scoped udev rule may
be preferable on managed systems.
:::

If capture works but activation does not, record a different chord. The
desktop environment may reserve the default.

## Global shortcut conflicts

Windows and macOS use the native global-shortcut registration path. Another
application or the operating system can own the same chord.

1. Open **Settings > General**.
2. Record a different chord containing Ctrl, Alt, or the platform Meta key.
3. Test it while onQ is hidden to the tray.

The in-window **Ctrl+Q** or **Cmd+Q** fallback only works while the app window
has keyboard focus.

## Search returns no result

1. Confirm the vault is unlocked.
2. Search for a full alphanumeric word from title, tag, project, or body.
3. Open Library and verify the prompt exists.
4. Save the prompt again to refresh its derived index row.
5. Check **Settings > Search**.

Palette text is not a filter language. `tag:writing` searches those literal
tokens; it does not apply a tag filter. Use a smart folder for structured
filters.

Locked bodies are absent from sparse search. Search their title, tags, or
project, or rely on a preserved semantic embedding.

## Search is sparse-only

This is the expected default before MiniLM is loaded.

1. Open **Settings > Search**.
2. Select **Load MiniLM model**.
3. Keep onQ open while the model downloads and every prompt is re-embedded.
4. Select **Refresh status**.

Keyword search remains fully usable without the model.

## MiniLM download fails

The explicit download needs HTTPS access to Hugging Face Hub. Check:

- internet connectivity;
- proxy and firewall rules for the launched onQ process;
- free space in the platform app cache;
- write permission to the app cache;
- system clock and TLS trust store.

The model directory must ultimately contain:

```text
all-MiniLM-L6-v2/model.onnx
all-MiniLM-L6-v2/tokenizer.json
```

Do not download random replacements. The code expects the exact
`sentence-transformers/all-MiniLM-L6-v2` ONNX graph and tokenizer.

## Model is cached but not loaded

Select **Refresh status**. onQ attempts to load both files from cache. If it
still reports **On disk (not loaded)**:

- verify both files are complete;
- remove only the model cache and use **Load MiniLM model** again;
- check runtime logs for an ONNX input or output-shape error.

Deleting the model cache does not delete prompts.

## Dense index stays pending

Dense mode can use exact cosine while a replacement index is pending. Keep the
vault open, wait for the rebuild, then refresh status. If it remains pending:

1. switch to Binary;
2. wait for that rebuild;
3. switch back to Dense;
4. inspect logs if either operation errors.

Binary is the safe default.

## Clipboard copy fails

Confirm that:

- the prompt is not locked;
- the operating system permits clipboard access;
- a remote or sandboxed desktop session exposes a clipboard;
- template input was not canceled.

Try the editor Copy action. It bypasses template substitution and helps
separate clipboard failures from template prompts.

## Import created nothing

The Settings picker expects a directory of Markdown files.

- Files must end in lowercase `.md`.
- Recursion stops after four nested levels.
- Existing IDs are skipped.
- Invalid files appear in the reported error count.
- JSON file selection is not exposed by the current picker.

Import a small directory with one plain Markdown file first.

## Backup export fails

- Confirm the selected destination is writable.
- Save outside the vault.
- Verify the open path is a valid vault with `.onq/salt` and
  `.onq/search-index/`.
- If archive protection is enabled, enter matching non-empty passwords.
- Check free disk space for a complete copy of the vault.

## Backup import fails

Messages have distinct meanings:

- `not an onQ backup (.onqbak) or unsupported version`: wrong file or header.
- `this backup is password-protected`: enter its archive password.
- `wrong archive password or corrupted backup`: wrong password or damaged
  ciphertext.
- `backup file is truncated`: incomplete copy.
- `missing .onq/salt or .onq/search-index`: payload is not a full vault.

Import replaces the current vault root. Keep another copy until the restored
vault unlocks and its prompts are verified.

## Update check fails

Manual update checks need access to the configured GitHub release URLs.
Developer builds and repositories without matching updater artifacts can
report feed errors. Startup checks intentionally hide those failures.

Download a release manually from:

```text
https://github.com/visorcraft/onQ/releases
```

The beta preference does not select a beta feed yet.

## Source build fails

Check versions first:

```bash
node --version
rustc --version
cargo --version
```

Node must be 24 or newer. Use Rust stable and install the platform Tauri
prerequisites.

Then run focused checks:

```bash
npm ci
npm run check
npm run build
cargo fmt --all -- --check
cargo check --workspace
```

ONNX Runtime artifacts are target-specific. The automated macOS release
currently targets Apple Silicon because the selected `ort` API package does
not provide the required Intel macOS download.

See [Build and test](/development) for the complete matrix.

## Documentation build fails

Run:

```bash
npm ci
npm run docs:build
```

VitePress reports broken internal links. Page links should be root-relative,
such as `/security`, because the configured `/onQ/` base is added at build
time.

## Ask for help

Search existing
[GitHub issues](https://github.com/visorcraft/onQ/issues), then open a focused
report with:

- onQ version;
- OS and CPU architecture;
- install type;
- exact error;
- minimal reproduction;
- relevant logs with secrets and local identifiers removed.

Report vulnerabilities privately through the
[security advisory form](https://github.com/visorcraft/onQ/security/advisories/new).
