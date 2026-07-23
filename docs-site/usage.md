---
title: Usage
---

# Usage

A quick tour of onQ's key features. Use **Win+Q** / **Meta+Q** / **⌘+Q** to open
the command palette — that is the app.

## Search

Search is the heart of onQ. Every keystroke in the command palette runs a hybrid
query that fuses two scoring streams:

- **Lexical** — normalized sparse term matching over prompt bodies.
- **Semantic** — cosine similarity over `all-MiniLM-L6-v2` embeddings of each prompt.

Results from both streams are merged via **Reciprocal Rank Fusion** with a recency-decay
boost so newer prompts surface higher when relevance is otherwise tied. Half-life is
tunable under **Settings → Search** (default 30 days).

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
- **Smart folders** are saved searches. Use the DSL editor (`favorite:true tag:writing`)
  or the visual predicate builder (chips for tag/folder/favorite/locked/text).

## Templates

Prompt bodies may include `{{name}}` or `{{name|default}}` placeholders. Copying from
the palette prompts for fill-in values when placeholders are present.

## Locked prompts

Each prompt can have its body encrypted with a per-prompt key stored in the OS keychain.
Locked prompts show title and metadata in search results, but the body stays sealed until
you unlock it (which prompts for the master passphrase).

## Three-way merge

onQ watches your vault directory and reconciles external edits automatically. If you
edit a prompt in Vim, the change is picked up the next time the file is saved. If two devices
edit the same prompt concurrently, a three-way merge runs against the last-known-good version.
Conflicts surface in a glass diff UI; nothing is silently overwritten.

## History

Each save writes a snapshot under `.onq/history/`. Open a prompt → History to browse
and restore prior bodies. Retention is configurable (default 30 days).

## Plugins

onQ supports signed Rust-native plugins via a C ABI. Install/manage them under
**Settings → Plugins**. See the [plugin authoring guide](/../plugins/) in the repo
(`docs/plugins/README.md`).

## Tag suggestions

When editing a prompt, onQ can suggest tags by matching body tokens against your
existing vault vocabulary (not a remote classifier). Accept with **Tab**, dismiss with **Esc**.

## Settings worth knowing

- **Settings → Search → Recency half-life** — how fast newer prompts outrank older ones. Default: 30 days.
- **Settings → Vault → Auto-lock** — idle timeout, lock on quit, or disabled. Idle tracking resets on activity.
- **Settings → Vault → History retention** — days of prompt snapshots kept.
- **Settings → Backups** — export/import `.onqbak`; optional backup-age reminders.
- **Settings → Updates → Beta channel** — opt into pre-release builds.
- **Palette → Lock vault** — clear the session immediately without quitting.

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
