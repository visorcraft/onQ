---
title: Palette and search
description: Palette behavior, keyword and semantic retrieval, ranking, model setup, and smart-folder filtering.
---

# Palette and search

The Palette is onQ's fastest path from intent to clipboard. It searches the
open vault and also exposes creation, lock, similarity, and plugin commands.

## Open the Palette

The default global shortcut is:

| Platform | Default |
|---|---|
| Windows | **Win+Q** |
| Linux | **Meta+Q** |
| macOS | **⌘+Q** |

The shortcut restores the window from the tray and toggles the Palette. Inside
the app, **Ctrl+Q** or **Cmd+Q** also toggles it. Record a different chord from
**Settings > General > Global shortcut**.

## Empty-query behavior

With no search text, the Palette shows:

1. **New prompt**
2. **Lock vault**
3. enabled plugin commands, when any exist
4. up to five recently copied or edited prompts
5. current similarity results, when a **More like this** lookup has run

Recent prompt IDs live in the local WebView profile. If no recent history
exists, onQ falls back to the five most recently updated vault prompts.

## Free-text search

Typing starts a debounced search after 150 ms. The searchable document for
each unlocked prompt combines:

- title;
- tags;
- project path;
- body.

The backend returns at most 50 results for the Palette. Results include title,
metadata, and a short body snippet. A locked result never includes a plaintext
body snippet.

The Palette input is free text. It does not parse `tag:`, `folder:`,
`recent:`, or `lock:` operators. Structured operators are only accepted by
the [smart-folder DSL](#smart-folder-dsl).

## Keyword retrieval

Keyword retrieval is always available. onQ:

1. splits text on non-alphanumeric characters;
2. lowercases each token;
3. hashes tokens into a deterministic sparse vector;
4. normalizes term frequencies so long prompts do not win only by length;
5. queries MongrelDB's sparse index.

This is token matching, not substring matching. Search for meaningful words
from a title, tag, project, or body.

## Optional semantic retrieval

Semantic search uses
`sentence-transformers/all-MiniLM-L6-v2` through ONNX Runtime:

- output dimension: 384;
- pooling: attention-mask-aware mean pooling;
- output normalization: L2;
- inference: local;
- download source: Hugging Face Hub;
- approximate model size: 90 MB.

The model is not downloaded on first launch. Select **Settings > Search > Load
MiniLM model**. onQ downloads `onnx/model.onnx` and `tokenizer.json` into the
platform app cache, loads the model, and re-embeds every prompt.

If the model is absent or fails to load, search continues in keyword-only
mode. Existing prompt creation and editing do not need network access.

## Ranking

When semantic search is available, onQ queries both:

- sparse keyword relevance;
- approximate nearest-neighbor vector similarity.

It combines emitted result ranks with Reciprocal Rank Fusion. Two small
tie-breakers then favor:

- recently updated prompts;
- favorites.

The **Search recency half-life** setting controls the exponential recency
decay period. The default is 30 days. Larger values preserve the recency boost
for longer; smaller values make it disappear faster.

Favorites receive only a small bonus. A strongly relevant non-favorite should
still outrank a weak favorite.

## Embedding index modes

After the model is loaded, **Settings > Search** offers:

| Mode | Behavior | Use when |
|---|---|---|
| Binary, default | Binary HNSW candidates followed by exact cosine reranking | Lower memory and fast retrieval |
| Dense | Full-precision dense ANN after index publication | Recall matters more than memory |

Changing mode persists the preference and rebuilds the derived embedding
index. Dense mode can use exact cosine temporarily while the replacement index
is pending.

Neither mode changes prompt files. The embedding index can be rebuilt.

## More like this

Open an existing prompt in the editor, then open the Palette again. The
**More like this: _title_** command becomes available.

This path uses MinHash over character trigrams, not MiniLM. It works without
the optional model and returns up to ten prompts with similar body shingles.
The source prompt is removed from its own results.

## Smart-folder DSL

Smart folders accept structured filters:

| Token | Meaning | Example |
|---|---|---|
| `text:<value>` | Add free text to ranked retrieval | `text:"release notes"` |
| `folder:<path>` | Exact project-path match | `folder:"Work/Releases"` |
| `tag:<tag>` | Add a tag to the tag-any filter | `tag:writing` |
| `-tag:<tag>` | Remove that tag from tags already named in this DSL | `-tag:draft` |
| `favorite:true` or `false` | Filter favorite state | `favorite:true` |
| `locked:true` or `false` | Filter lock state | `locked:false` |
| `char:>N` | Minimum character count | `char:>500` |
| `char:<N` | Maximum character count | `char:<5000` |

Values with spaces may use single or double quotes. Multiple `text:` values
join with spaces. Multiple positive `tag:` values use any-tag matching, not
all-tag matching.

`-tag:` is not a general exclusion operator. It only removes a matching
positive tag token from the query being assembled. For example,
`tag:draft -tag:draft` leaves no tag filter.

Unknown operators are rejected. The DSL cannot run commands or scripts.

Examples:

```text
favorite:true tag:writing
```

```text
text:"incident response" folder:Operations locked:false
```

```text
char:>200 char:<4000
```

The visual builder can create tag, folder, favorite, locked, and text
predicates. Character bounds require direct DSL editing.

## Search and locked prompts

Locking a prompt removes its body from the Markdown file and sparse index.
Title, project, tags, favorite state, timestamps, and the lock marker remain
visible. A previously generated semantic embedding is preserved in the
protected search index, so the prompt may still rank semantically while
locked.

Copy and edit actions refuse to expose the locked body. Unlock the prompt from
its editor first.

## Relevance tips

- Put the words you normally recall in the title.
- Use a few stable tags rather than many near-duplicates.
- Use project paths for ownership or workflow, not every topic.
- Favorite high-value prompts.
- Load MiniLM only when semantic recall is useful enough to justify model
  storage and re-embedding time.
- Use smart folders for exact metadata filters. Keep Palette input as ordinary
  search text.

## Diagnose search

Open **Settings > Search** and inspect:

- keyword index status;
- model loaded, cached, or not installed;
- sparse-only or semantic path;
- binary/dense mode and readiness.

Use **Refresh status** after changing model files or waiting for a rebuild.
See [Troubleshooting](/troubleshooting) when status does not change.
