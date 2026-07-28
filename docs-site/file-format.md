---
title: Prompt file format
description: Canonical Markdown and YAML frontmatter format, field semantics, templates, locked files, and safe manual editing.
---

# Prompt file format

Each prompt is a UTF-8 Markdown file:

```text
<vault>/prompts/<id>.md
```

The filename uses the same 26-character ID stored in frontmatter.

## Complete example

```markdown
---
id: 01J3F0M4YZ7Q8P9R2S6T1V5WXC
title: Release checklist
folder: Work/Releases
tags:
- release
- checklist
favorite: true
locked: false
created: 2026-07-27T15:00:00Z
updated: 2026-07-27T16:30:00Z
---

Prepare release notes for {{audience|users}}.

Include:

- highlights
- fixes
- upgrade notes
```

The opening delimiter must start the file, except that a UTF-8 BOM is
accepted. The closing delimiter is another `---` line.

## Frontmatter fields

| Field | Type | Required | Meaning |
|---|---|---|---|
| `id` | string | Yes | Prompt identity and filename stem |
| `title` | string | Yes | Display title and searchable text |
| `folder` | string or omitted | No | Logical project path |
| `tags` | string array | No, defaults empty | Free-form tag metadata |
| `favorite` | boolean | No, defaults false | Favorite view and ranking hint |
| `locked` | boolean | No, defaults false | Whether body lives in a lock envelope |
| `created` | RFC 3339 timestamp | Yes | Creation time |
| `updated` | RFC 3339 timestamp | Yes | Last update time |

onQ-generated IDs are monotonic ULIDs. The parser accepts a 26-character
ASCII alphanumeric ID.

Timestamps are generated in UTC. Preserve timezone information when editing
manually.

## Body

Everything after closing frontmatter is the prompt body. onQ stores it as a
string and preserves line structure.

The editor's Preview tab is currently line-preserving, not a full Markdown to
HTML renderer. The file itself remains normal Markdown for other tools.

## Project paths

`folder` is a logical label:

```yaml
folder: Writing/Blog Posts
```

It does not map to a directory under `prompts/`. Library interprets `/` as
hierarchy. See [Library and organization](/library#projects) for validation
rules.

## Tags

Block and inline YAML arrays are both accepted by `serde_yaml`, but onQ renders
its canonical form on save. Tags are indexed as text and shown in Library.

Example input:

```yaml
tags: [writing, review]
```

After an onQ save, formatting may be normalized.

## Templates

Template syntax lives only in the body:

```text
{{name}}
{{name|default}}
```

Fields are resolved when copied from the Palette. YAML is never templated.
Escape literal braces with `\{\{` and `\}\}`.

See [Editing and templates](/editing#templates).

## Locked prompt file

A locked prompt keeps metadata but has an empty body:

```markdown
---
id: 01J3F0M4YZ7Q8P9R2S6T1V5WXC
title: Sensitive checklist
folder: Work
tags:
- private
favorite: false
locked: true
created: 2026-07-27T15:00:00Z
updated: 2026-07-27T16:30:00Z
---

```

The encrypted body is stored separately at:

```text
.onq/locked/01J3F0M4YZ7Q8P9R2S6T1V5WXC.enc
```

Do not set `locked` manually. The lock action must create the envelope and
keychain entry as one workflow.

## Plain Markdown import

The importer accepts `.md` without onQ frontmatter:

```markdown
# Incident response

Collect logs and summarize the timeline.
```

For that file, import:

- generates an ID;
- uses the filename stem as title;
- leaves project empty;
- leaves tags empty;
- sets favorite and locked false;
- sets current creation and update timestamps;
- keeps the entire original text as body.

## Safe manual editing

The live file watcher and conflict-resolution UI are not wired into the
desktop shell yet. For manual changes:

1. Quit onQ from the tray.
2. Back up the vault.
3. Edit files under `prompts/`.
4. Preserve filename, `id`, required fields, and valid YAML.
5. Do not edit `.onq/search-index/`.
6. Reopen onQ and verify the prompt.

Bulk imports are safer than inventing IDs by hand.

## Atomic writes

onQ renders the complete file to a sibling `.tmp` path, then renames it over
the final `.md`. Temporary files left by an interrupted filesystem operation
are not prompt source and should be inspected before deletion.

## Compatibility

The application is on the 1.x line. Keep full backups before large upgrades.
Schema migrations update the derived search index, but Markdown remains the
portable source format.
