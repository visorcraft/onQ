---
title: Editing and templates
description: Prompt drafts, metadata, preview, history, tag suggestions, locks, and template substitution.
---

# Editing and templates

Open the editor from a Palette pencil action, a Library row, or **New prompt**.

## Draft lifecycle

A new prompt is an in-memory draft:

- the title starts empty;
- the body starts empty;
- the project is pre-filled only when creation starts inside a project;
- canceling or pressing Escape discards it;
- the vault is not changed until **Save**.

On first save, onQ creates a 26-character ULID and uses `Untitled` when the
trimmed title is empty.

## Editable fields

| Field | Stored as | Notes |
|---|---|---|
| Title | YAML `title` | Included in search |
| Project path | YAML `folder` | Logical slash-separated path |
| Tags | YAML `tags` | Included in search and Library tag views |
| Favorite | YAML `favorite` | Adds Library view and small search bonus |
| Body | Markdown after frontmatter | Included in search while unlocked |
| Locked state | YAML `locked` | Managed by the lock action |

Created and updated timestamps are managed by onQ.

## Edit and preview

The body editor uses a line-numbered textarea. The Preview tab shows a
line-preserving read-only view. It does not currently render full Markdown
syntax into rich HTML.

Markdown remains the storage format, so external tools can render the file
normally.

## Save behavior

Saving:

1. creates the prompt on first save;
2. normalizes the title fallback and project path;
3. writes YAML frontmatter and body atomically through a temporary file;
4. writes a body snapshot under `.onq/history/`;
5. prunes snapshots older than the configured retention;
6. updates sparse search data;
7. updates the semantic embedding when the local model is loaded.

The editor closes after a successful save and Library refreshes.

## Copy behavior

The editor **Copy** action copies the body exactly as stored.

Selecting a prompt's main Palette row also copies its body. The Palette copy
path additionally resolves template variables. If **Automatically minimize
after clicking search result** is enabled, onQ hides to the tray after the
copy succeeds.

## Templates

Use double braces in a body:

```text
Write a {{tone}} response for {{audience|technical readers}}.
```

Supported forms:

```text
{{name}}
{{name|default value}}
```

When copied from the Palette:

1. fields are discovered in first-appearance order;
2. duplicate field names are asked once;
3. a dialog asks for each value;
4. a default pre-fills the answer when present;
5. canceling any field cancels the entire copy;
6. values are substituted only in the clipboard result.

The stored body is not changed.

Without a supplied value, the renderer uses the default or an empty string.
Whitespace around field names and defaults is trimmed. Only the first `|`
separates the name from its default.

Escape literal braces with backslashes:

```text
\{\{not a variable\}\}
```

This renders as <code v-pre>{{not a variable}}</code>.

::: info
Template substitution happens only when copying from the Palette. The editor
Copy action copies the raw body, including placeholders.
:::

## History

Each save writes a body snapshot under:

```text
.onq/history/<prompt-id>/<RFC3339 timestamp>.md
```

Open an existing prompt and select **History** to list snapshots. **Restore**
replaces the current body with the selected snapshot and writes the restored
state back through the normal vault path.

History retention defaults to 30 days and accepts 0 through 3650 days in
**Settings > Vault**. A value of 0 removes older snapshots during subsequent
writes.

History stores bodies, not complete frontmatter versions.

## Prompt locks

The lock action is available only after a draft has been saved.

Locking:

1. saves the current plaintext state;
2. creates a fresh 256-bit per-prompt key;
3. encrypts the body with AES-256-GCM;
4. writes `.onq/locked/<prompt-id>.enc`;
5. stores the body key in the OS keychain;
6. clears the body from the Markdown file;
7. marks `locked: true`.

The editor closes so every surface reloads the sealed state.

While locked:

- the body is read-only and hidden;
- editor and Palette copy are blocked;
- metadata remains visible;
- history restore is disabled.

Unlocking reads the per-prompt key from the OS keychain, restores the body to
the Markdown file, removes the envelope, and deletes the per-prompt keychain
entry.

Prompt locks depend on the keychain entry. They do not use the vault master
password or 24-word vault recovery phrase. Include `.onq/locked/` in backups,
but understand that the per-prompt key itself is held outside the vault.

## Delete behavior

Delete requires confirmation and then removes the current prompt. onQ has no
trash or undo command. Export backups before bulk cleanup.
