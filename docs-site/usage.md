---
title: Usage
---

# Usage

A quick tour of onQ's key features. Use **Ctrl+Q / ⌘Q** to open
the command palette — that is the app.

## Search

Search is the heart of onQ. Every keystroke in the command palette runs a hybrid
query that fuses two scoring streams:

- **Lexical** — normalized sparse term matching over prompt bodies.
- **Semantic** — cosine similarity over `all-MiniLM-L6-v2` embeddings of each prompt.

Results from both streams are merged via **Reciprocal Rank Fusion** with a recency-decay
boost so newer prompts surface higher when relevance is otherwise tied.

### Search syntax

| Prefix  | Meaning                                                      |
|---------|--------------------------------------------------------------|
| (none)  | Full-text query, hybrid mode                                 |
| `tag:`  | Restrict to a single tag (e.g. `tag:writing`)                |
| `folder:` | Restrict to a folder (e.g. `folder:projects/acme`)         |
| `recent:` | Limit to prompts modified within N days (e.g. `recent:7`)  |
| `lock:` | Filter locked/unlocked (e.g. `lock:locked`, `lock:unlocked`) |

Combine freely: `tag:writing recent:30 alpha launch`.

## Folders, tags, and smart folders

- **Folders** mirror your filesystem. Drag a prompt onto a folder in the sidebar to move
  its `.md` file.
- **Tags** are free-form strings in the YAML frontmatter. Multi-tag queries are AND-joined.
- **Smart folders** are saved searches. Build them with the visual query builder, or drop
  into the DSL editor for power-user queries.

## Locked prompts

Each prompt can have its body encrypted with a per-prompt key stored in the OS keychain.
Locked prompts show title and metadata in search results, but the body stays sealed until
you unlock it (which prompts for the master passphrase).

This is useful for prompts that contain API keys, internal URLs, or any other secret you
do not want sitting in plaintext on disk.

## Three-way merge

onQ watches your vault directory and reconciles external edits automatically. If you
edit a prompt in Vim, the change is picked up the next time the file is saved. If two devices
edit the same prompt concurrently, a three-way merge runs against the last-known-good version.
Conflicts surface in a glass diff UI; nothing is silently overwritten.

## Plugins

onQ supports Rust-native plugins via a C ABI. Plugins can:

- Add new sidebar panels.
- Register new commands in the palette.
- Define custom DSL operators for smart folders.
- Provide alternative embedding models.

All plugins must be signed with an ed25519 key registered with the project. Manifests are
verified on load; unsigned or tampered plugins are quarantined.

See the [plugin authoring guide](https://github.com/visorcraft/onQ/tree/main/docs/plugins)
(TBD) for the full SDK.

## AI tag-suggest

When you save a new prompt, onQ can suggest tags based on its content using a local
zero-shot classifier. No data leaves your machine. Suggestions appear in the prompt editor's
metadata panel — accept with **Tab**, dismiss with **Esc**.

## Settings worth knowing

- **Settings → Search → Recency decay** — controls how aggressively newer prompts outrank
  older ones with the same relevance score. Default: 30 days.
- **Settings → Encryption → Auto-lock after** — minutes of inactivity before the vault
  re-locks. Default: 15.
- **Settings → Updates → Beta channel** — opt into pre-release builds.
- **Settings → Vault → Vault directory** — move your vault to a synced folder.

## Keyboard shortcuts

| Shortcut         | Action                          |
|------------------|---------------------------------|
| **Ctrl/Cmd + Q** | Open command palette            |
| **Ctrl/Cmd + N** | New prompt                      |
| **Ctrl/Cmd + ,** | Open settings                   |
| **Ctrl/Cmd + /** | Toggle sidebar                  |
| **Esc**          | Close palette / cancel edit     |
| **Tab**          | Accept tag suggestion           |
| **?**            | Show all shortcuts              |
