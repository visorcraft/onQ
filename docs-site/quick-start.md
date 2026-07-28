---
title: Quick start
description: Create a vault, save a prompt, search it, organize it, and make a backup.
---

# Quick start

This path verifies the core onQ workflow in a few minutes.

## 1. Create or open a vault

On first launch, choose:

- **Create new vault** to initialize a selected directory.
- **Open existing vault** to unlock a directory that already contains an
  onQ vault.
- A path under **Recent vaults** to reopen or recover one used before.

Creating a vault adds `prompts/` and `.onq/` under the selected directory.
onQ refuses creation when that directory already contains
`.onq/search-index`.

## 2. Choose authentication

After selecting a new vault directory, choose one mode:

| Mode | What you provide | What onQ stores | Recovery |
|---|---|---|---|
| Master password | A password and confirmation | Per-vault salt and auth-mode marker; not the password | None |
| No password | Nothing | A generated secret in the operating-system keychain | One 24-word recovery phrase |

For a no-password vault, onQ requires you to copy the recovery phrase and
confirm that it has been saved before continuing.

::: warning
A password vault cannot be recovered if its password is lost. A keychain
vault cannot be recovered if both its keychain entry and 24-word phrase are
lost.
:::

Read [Vault security and recovery](/security) before storing sensitive data.

## 3. Create a prompt

1. Open the Palette from the Home card or global shortcut.
2. Select **New prompt**.
3. Enter a title and body.
4. Optionally set a project path, tags, and favorite status.
5. Select **Save**.

A new editor is only a draft. Canceling or pressing Escape before **Save**
writes nothing to the vault.

## 4. Search and copy

1. Return Home.
2. Press **Win+Q** on Windows, **Meta+Q** on Linux, or **⌘+Q** on macOS.
3. Type part of the prompt title, tag, project path, or body.
4. Select the main result row.

The prompt body is copied to the clipboard and the Palette closes. Use the
pencil action when you want to edit instead.

Keyword search is available immediately and stays local. Optional semantic
search is configured later.

## 5. Organize the prompt

Open **Library** and try:

- selecting **Favorites**, **Recent**, or **Unfiled**;
- creating a project;
- moving the prompt into that project;
- selecting a tag from the tag list;
- creating a smart folder such as `favorite:true`.

Projects are metadata labels, not physical subdirectories. All prompt files
remain under `prompts/`.

## 6. Create a template

Edit the body to contain:

```text
Write a {{tone|concise}} summary for {{audience}}.
```

Copy that prompt from the Palette. onQ asks for each unique field, substitutes
the values, and copies the rendered body. The stored prompt remains unchanged.

## 7. Make a backup

1. Open **Settings > Backups**.
2. Select **Export backup**.
3. Turn on archive password protection if the backup may leave trusted
   storage.
4. Save the `.onqbak` file somewhere outside the vault.

An unsealed full backup contains the entire vault tree, including plaintext
bodies of unlocked prompts. See [Backups and portability](/backups).

## 8. Optional: enable semantic search

1. Open **Settings > Search**.
2. Select **Load MiniLM model**.
3. Wait for the download and vault re-embedding to finish.
4. Select **Refresh status**.

This is the only prompt-search feature that needs a model download. The model
comes from Hugging Face and is about 90 MB. Prompt text is embedded locally
after download.

## Next steps

- Learn ranking and model behavior in [Palette and search](/search).
- Learn projects and smart folders in [Library and organization](/library).
- Understand plaintext files and prompt locks in [Vault security and
  recovery](/security).
- Review current product boundaries in [Feature status](/feature-status).
