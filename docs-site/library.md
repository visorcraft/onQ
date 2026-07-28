---
title: Library and organization
description: Browse prompts, use projects and tags, build smart folders, and understand delete and rename behavior.
---

# Library and organization

Library organizes prompt metadata without changing the portable file model.
All prompt Markdown files stay under `prompts/`.

## Built-in views

| View | Contents |
|---|---|
| All prompts | Every readable prompt file |
| Favorites | Prompts with `favorite: true` |
| Recent | The 30 most recently updated prompts |
| Unfiled | Prompts with no project path |
| Project | Prompts in that project or any descendant project |
| Smart folder | Results from a saved smart-folder DSL query |
| Tag | Prompts containing that exact tag |

All views except Recent sort newest first. Lists use pages of 25 prompts.

The **Filter this list** field is local to the current Library selection. It
matches title, visible body preview, tag, or project path, case-insensitively.
It is not the hybrid Palette search.

## Projects

Projects are slash-separated logical paths stored in each prompt's `folder`
frontmatter field:

```text
Writing/Blog Posts
```

The separate folders table also records registered paths so an empty project
can exist. Library synthesizes missing ancestors from prompt metadata so the
tree remains complete.

Project rules:

- `/` is the hierarchy separator.
- Leading, trailing, and repeated separators are normalized.
- Whitespace around each path segment is trimmed.
- `.` and `..` segments are rejected.
- Quotes, NUL, and control characters are rejected.
- Maximum depth is eight segments.
- Maximum normalized path length is 200 bytes.

### Create a project

Select the plus action beside **Projects**, enter a root name, and create it.
Use the plus action on an existing project to add a direct child.

Creating a prompt while a project is selected pre-fills that project path.

### Move a prompt

Use the move action on a Library row and select:

- another registered project; or
- **Unfiled** to clear the project path.

Moving updates frontmatter. It does not move the Markdown file.

### Rename a project

Rename accepts only the leaf name. Renaming a project:

- rewrites that path and all descendant project registrations;
- rewrites matching prompt project paths;
- rewrites `folder:` operands in smart folders under the renamed path.

### Delete a project

Deleting a project also removes descendant project registrations. Prompts are
not deleted. Prompts under the removed tree become Unfiled.

Matching `folder:` tokens are removed from saved smart folders. If that would
leave a smart folder with no filters, onQ replaces its query with
`char:<-1`, an intentional match-nothing query.

## Tags

Tags are free-form strings stored in YAML frontmatter. In the editor:

- Enter or comma commits the current text;
- comma-separated text can add several tags;
- duplicate tags are prevented case-insensitively;
- Backspace on an empty tag input removes the last tag;
- the dropdown offers tags already used in the vault.

The Library tag list is built from current prompt metadata and shows prompt
counts. Tag-view matching uses the stored tag text.

## Local tag suggestions

As an unlocked prompt body changes, onQ compares its tokens with the vault's
existing tag vocabulary. Suggestions are local heuristics, not a remote
classifier.

- **Tab** accepts the first suggestion.
- **Escape** dismisses suggestions.

Suggestions never add a tag without an explicit acceptance.

## Favorites

Select the star in the editor or on a Library row. Favorites:

- appear in the Favorites view;
- can be selected by `favorite:true` smart folders;
- receive a small search-ranking bonus.

Favorite state is ordinary frontmatter and remains portable.

## Smart folders

A smart folder is a named saved query. It does not contain or move prompts.
Deleting it removes only the saved query.

Create one with:

1. a name;
2. visual filter chips, direct DSL text, or both;
3. **Create**.

The visual builder supports:

- tag is / tag not;
- folder is;
- favorite is;
- locked is;
- text contains.

The direct DSL additionally supports character-count bounds. See the complete
[smart-folder DSL reference](/search#smart-folder-dsl).

### Useful examples

Favorite writing prompts:

```text
favorite:true tag:writing
```

Unlocked prompts in an exact project:

```text
folder:"Work/Support" locked:false
```

Prompts about releases with a bounded body size:

```text
text:"release" char:>200 char:<5000
```

## Prompt row actions

Select the main row to edit. Adjacent actions:

- toggle favorite;
- move to a project or Unfiled;
- delete after confirmation.

Delete removes the current prompt file and index row. There is no trash UI.
Treat deletion as permanent and keep current backups.

## Locked rows

Locked prompts keep visible metadata but show no body preview or character
count. Open the prompt and unlock it before editing, copying, or restoring
history.
