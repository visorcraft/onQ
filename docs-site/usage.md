---
title: Feature guide
description: A map of onQ user-facing features and where to learn each workflow.
---

# Feature guide

onQ has four main surfaces: Home, Palette, Library, and Settings. Prompt editing
opens as a modal from the Palette or Library.

## Home and app chrome

After a vault opens, Home shows the configured global shortcut. The controls in
the upper area open:

- **Home**
- **Library**
- **Settings**
- **Theme**
- **About**, Licenses, and Credits

Closing or minimizing the desktop window hides onQ to the system tray. Use the
tray icon, tray menu, or global shortcut to show it again. Choose **Quit** from
the tray menu to exit the process.

The version button at the bottom of Home performs a manual update check.

## Palette

Press **Win+Q**, **Meta+Q**, or **⌘+Q** from anywhere, or click the Home card.
The Palette:

- searches title, tags, project path, and body;
- shows up to five recently copied or edited prompts when the query is empty;
- copies a result's body as its primary action;
- opens the editor from the pencil action;
- creates a draft with **New prompt**;
- locks the current vault with **Lock vault**;
- shows commands registered by enabled plugins.

Typing `tag:`, `folder:`, or similar prefixes into the Palette does not create
structured filters. Those operators belong to the smart-folder DSL. See
[Palette and search](/search).

## Library

Library is the browsing and organization view. It contains:

- all prompts, favorites, recent prompts, and unfiled prompts;
- nested project paths;
- smart folders;
- tag views;
- local list filtering;
- prompt favorite, move, edit, and delete actions.

Projects are logical slash-separated labels stored in frontmatter. They do not
move prompt files into matching filesystem directories. See
[Library and organization](/library).

## Editor

The editor supports:

- title, body, project path, tags, and favorite metadata;
- edit and line-preserving preview modes;
- tag completion and local tag suggestions;
- copy, save, and delete;
- prompt locking and unlocking;
- body history and restore;
- templates resolved when copied from the Palette.

New prompts remain unsaved drafts until **Save** is selected. See
[Editing and templates](/editing).

## Settings

Settings is divided into:

- **General:** global shortcut, minimize after copy, theme, and language.
- **Search:** optional MiniLM model, embedding index mode, model status,
  embedder preference, and recency half-life.
- **Vault:** auto-lock, history retention, Markdown import/export, audit log,
  and authentication information.
- **Backups:** vault paths and full `.onqbak` export/import.
- **Plugins:** install, enable, disable, refresh, and uninstall.
- **Updates:** stable update check and the stored beta preference.

See the complete [Settings reference](/settings).

## Keyboard controls

| Context | Control | Action |
|---|---|---|
| System-wide | **Win+Q**, **Meta+Q**, or **⌘+Q** | Show onQ and toggle the Palette |
| App window | **Ctrl+Q** or **Cmd+Q** | Toggle the Palette |
| Palette | **Arrow Up / Arrow Down** | Move through actions and results |
| Palette input | **Enter** | Run the first visible action or result |
| Palette or editor | **Escape** | Close the active surface |
| Tag input | **Enter** or comma | Commit tags |
| Tag input | **Backspace** on empty input | Remove the last tag |
| Tag suggestion | **Tab** | Accept the first suggestion |
| Tag suggestion | **Escape** | Dismiss suggestions or the tag menu |
| Tutorial | **Arrow Left / Arrow Right** | Move between tutorial steps |

The global shortcut is user-recordable. It must include Ctrl, Alt, or the
platform Meta key plus another supported key. Linux global capture also needs
read access to keyboard input devices. See [Troubleshooting](/troubleshooting).

## Current boundaries

Live filesystem watching, automatic three-way conflict handling, beta-feed
routing, alternate plugin embedders, and the full native plugin callback
surface have code or UI scaffolding but are not complete end-to-end product
paths. See [Feature status](/feature-status) before relying on them.
