# Complete Product Gap Closure — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` per **epic**, not as one mega-session. Each epic below becomes a kickoff-branch + TDD cycle. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close every documented/product gap between what onQ claims (or a competitive local-first prompt vault needs) and what ships today — without abandoning local-first, GPL-3.0-only, or Markdown-as-source-of-truth.

**Architecture:** Ship in **seven dependency-ordered waves**. Wave 0–2 finish half-built surfaces that already have core code. Wave 3–4 add daily-driver product features. Wave 5 expands platform/extensibility. Wave 6 is polish and 1.0 freeze. Prefer **wiring existing modules** over greenfield rewrites; never introduce cloud accounts, telemetry, or non-GPL plugin licenses.

**Tech Stack:** Tauri 2 + Svelte 5 + TypeScript UI; Rust (`onq-core`, `onq-app`, `onq-plugin-sdk`); MongrelDB `mongreldb-core` 0.64.x (encrypted); MiniLM ONNX embeddings; Vitest / Playwright / `cargo test`.

## Global Constraints

- **Local-first:** zero remote calls by default; Hugging Face model download remains the only optional network path.
- **Vault model:** prompts stay portable `.md` + YAML frontmatter; index under `.onq/search-index` stays encrypted.
- **License:** GPL-3.0-only app and plugins; no proprietary SDK surface.
- **No AI attribution** in commits, PRs, code comments, or test data.
- **Conventional Commits** per change; each epic merges via PR when possible.
- **Verify before “done”:** every task ends with a runnable command that must exit 0.
- **TDD:** public behavior gets a failing test first (Rust unit/integration or Vitest).
- **Pre-alpha → 1.0:** Waves 0–4 target a usable 1.0; Waves 5–6 may ship as 1.x minors if scope threatens quality.
- **Do not adopt `mongreldb-kit` in this plan** unless a later epic explicitly revisits schema/migrations (out of scope here).

---

## Decision defaults (locked for this plan)

| Topic | Default | Rationale |
|-------|---------|-----------|
| Template syntax | `{{name}}` and `{{name\|default}}`; optional `?` interactive fill on copy | Familiar, grep-friendly, works in plain Markdown |
| Multi-vault | Recent-vault list (paths) + switcher; one open vault at a time | Matches existing `close_vault` / single `Db` session |
| Tag suggest | Wire **existing** vocab matcher first; optional better model later as plugin | Docs over-claimed; ship truth then improve |
| Auto-lock | Idle timer on window focus/keyboard/pointer; lock = `close_vault` + unlock UI | Policy already exists; activity was the gap |
| History UI | List snapshots → DiffViewer → restore (write file + reindex) | Snapshots already on disk |
| Visual smart folders | Structured JSON ↔ DSL bidirectional; simple chip/predicate UI | Column `query_visual` already in schema |
| Plugins | Settings UI + docs; HostApi command registration surfaces in palette | ABI exists; UI/docs missing |
| Embedding models | Built-in MiniLM remains default; plugins may register alternate embedders | Avoid multi-model download UX in 1.0 core |
| Import | First-party importers: plain folder of `.md`, JSON array, ChatGPT/Claude export heuristics | Best bang for switchers |
| Mobile / i18n / team collab | Wave 6 / post-1.0 unless explicitly pulled forward | High cost; not required for desktop 1.0 |
| Collaboration | File-sync only (existing three-way merge); no multiplayer | Local-first identity |

---

## Dependency graph

```text
Wave 0  Foundation hardening (session lock, settings truth, docs honesty)
   │
   ├─► Wave 1  Security & history productization
   │      (auto-lock live, lock-now, history UI, retention setting)
   │
   ├─► Wave 2  Search & organization completeness
   │      (tag suggest UI, recency decay setting, visual smart folders,
   │       body sparse/minhash writers, search highlight polish)
   │
   ├─► Wave 3  Daily launcher power
   │      (template variables, fill-on-copy, multi-vault switcher)
   │
   ├─► Wave 4  Migration & portability
   │      (bulk import, curated export, scheduled backup reminders)
   │
   ├─► Wave 5  Extensibility & depth
   │      (plugin manager UI, authoring guide, embedder plugins,
   │       editor Markdown preview, optional model picker hooks)
   │
   └─► Wave 6  1.0 polish & stretch
          (i18n scaffold, audit log, per-folder profiles,
           browser companion spike OR defer, mobile defer)
```

**Parallelism:** Within a wave, tasks marked `(parallel-ok)` may run in parallel worktrees. Across waves, only start the next wave after the previous wave’s **merge criteria** pass.

---

## Current baseline (do not re-implement)

| Module | Path | Already provides |
|--------|------|------------------|
| Auto-lock policy | `crates/onq-app/src/auto_lock.rs` | `AutoLockPolicy`, `should_lock_now` |
| Session close | `crates/onq-app/src/state.rs` | `close_vault()` |
| History IO | `crates/onq-core/src/history.rs` | `snapshot`, `list_snapshots`, `prune_older_than` |
| Tag suggest core | `crates/onq-core/src/tag_suggest.rs` | `suggest_tags(body, vocabulary, max_n)` |
| Smart folder DSL | `crates/onq-core/src/smart_folder_dsl.rs` | `parse` / serialize tokens |
| Smart folder CRUD | `crates/onq-app/src/commands.rs` | CRUD + `query_visual` column support |
| Diff UI | `src/lib/components/DiffViewer.svelte` | Three-way / side-by-side ready |
| Plugins install | `crates/onq-core/src/plugin_install.rs`, commands | install / list / enable / uninstall |
| Plugin ABI | `crates/onq-plugin-sdk` | `HostApi`, `export_plugin!` |
| Backup | `crates/onq-core/src/backup/`, Settings → Backups | `.onqbak` export/import |
| Search | `crates/onq-core/src/search.rs` | Hybrid RRF + fixed 30-day recency exp |
| Embeddings | `crates/onq-core/src/embed.rs` | `MODEL_ID` MiniLM-L6-v2 |
| Copy path | `src/lib/components/Palette.svelte` | Clipboard + minimize-on-copy |

---

# Wave 0 — Foundation & honesty

**Merge criteria:** Docs match product; lock-now works; settings for auto-lock and recency exist even if idle tracking lands in Wave 1; CI green.

### Task 0.1: Docs honesty pass

**Files:**
- Modify: `docs-site/usage.md`, `README.md`, `docs-site/index.md`
- Create: `docs/plugins/README.md` (stub pointing to Wave 5 full guide)

**Scope:**
- Tag-suggest: document as “vocabulary-based suggestions” until Wave 2 ships UI; remove “zero-shot classifier” claim until true.
- Plugins: “installable via Settings (Wave 5)” or “CLI/commands available; UI forthcoming” — pick one after 0.2–0.3 if UI slips.
- Auto-lock: “idle auto-lock in progress” until Wave 1 merges; do not claim full idle lock.
- Plugin authoring: replace `(TBD)` with real path under `docs/plugins/`.

- [ ] **Step 1:** Inventory every claim in `docs-site/usage.md` against code; rewrite mismatches.
- [ ] **Step 2:** `npm run docs:build` or site build script if present; else visual review.
- [ ] **Step 3:** Commit `docs: align product claims with shipped behavior`

### Task 0.2: `lock_vault_now` command + palette action

**Files:**
- Modify: `crates/onq-app/src/commands.rs`, `crates/onq-app/src/lib.rs` (invoke handler list)
- Modify: `crates/onq-app/src/state.rs` (reuse `close_vault`)
- Create: `src/lib/api/session.ts` — `lockVaultNow()`
- Modify: `src/lib/components/Palette.svelte` — command “Lock vault”
- Modify: `src/App.svelte` — handle session-closed → unlock UI
- Test: `crates/onq-app` unit test around close; Vitest for palette command registration

**Behavior:**
- `lock_vault_now` → `state.close_vault()`, clear in-memory passphrase/keys, emit event or return status so UI shows `VaultUnlock`.
- Does not quit the app; does not delete vault files.
- Tray menu: “Lock vault” entry.

- [ ] **Step 1:** Rust test: open temp vault → `close_vault` → subsequent list/search fails with locked error.
- [ ] **Step 2:** Implement command + frontend wiring.
- [ ] **Step 3:** Verify: `cargo test -p onQ lock_vault` (or named module tests) && targeted Vitest.
- [ ] **Step 4:** Commit `feat: lock vault now from palette and tray`

### Task 0.3: Settings surface for auto-lock + recency (UI + persistence)

**Files:**
- Modify: `src/lib/components/SettingsPage.svelte` — Vault section: auto-lock policy; Search section: recency half-life days
- Modify: `src/lib/stores/settings.ts` (or new stores)
- Modify: `crates/onq-app/src/commands.rs` — ensure `set_auto_lock_policy` / `get_auto_lock_policy` used from UI; add `search_recency_days` app_setting
- Modify: `crates/onq-core/src/schema.rs` / migrate if new app_state column needed — prefer reusing free-form settings via `get_app_setting` / `set_app_setting` keys: `auto_lock_policy`, `search_recency_days`, `history_retention_days`

**Defaults:**
- `auto_lock_policy`: `idle_timeout:900` (15 min) or existing default
- `search_recency_days`: `30` (matches current hard-coded `30.0` in `search.rs`)
- `history_retention_days`: `30` (matches `prune_older_than(..., 30)`)

- [ ] **Step 1:** Vitest: load/save settings round-trip with mocked invoke.
- [ ] **Step 2:** Wire UI controls.
- [ ] **Step 3:** Commit `feat: settings for auto-lock policy, recency, history retention`

### Task 0.4: Parameterize search recency from setting

**Files:**
- Modify: `crates/onq-core/src/search.rs` — `score_hit` / fusion takes `recency_half_life_days: f64`
- Modify: `crates/onq-app/src/commands.rs` `search` — read setting, pass through
- Test: update `search.rs` unit tests for half-life sensitivity

- [ ] **Step 1:** Failing unit test with half-life 1 vs 365 changes ordering/boost.
- [ ] **Step 2:** Implement parameter; default 30 when unset.
- [ ] **Step 3:** `cargo test -p onq-core recency`
- [ ] **Step 4:** Commit `feat: configurable search recency half-life`

**Wave 0 verify:**  
`cargo test -p onq-core && cargo test -p onQ && npm test -- --run`

---

# Wave 1 — Security & history productization

**Depends on:** Wave 0 (`lock_vault_now`, settings keys).

### Task 1.1: Activity tracking + idle auto-lock timer

**Files:**
- Modify: `crates/onq-app/src/auto_lock.rs` — keep pure `should_lock_now`
- Modify: `crates/onq-app/src/state.rs` — `last_activity: Mutex<Instant>`, `touch_activity()`, policy storage
- Modify: `crates/onq-app/src/lib.rs` / `commands.rs` — `touch_activity` command; background tick (Tokio interval or frontend poll)
- Modify: `src/App.svelte` — window listeners (`pointerdown`, `keydown`, `focus`) call `touch_activity`; on lock event → unlock UI
- Prefer: frontend interval every 15s invokes `evaluate_auto_lock` to avoid OS-level hooks complexity on Linux

**Behavior:**
- IdleTimeout: if `now - last_activity >= duration` → same path as `lock_vault_now`.
- LockOnQuit: lock on app exit only (already partial via `apply_auto_lock_on_start` semantics — clarify and implement quit hook).
- Disabled: no-op.
- Activity reset on successful unlock.

- [ ] **Step 1:** Unit tests for policy already exist; add integration-style test with mocked instants if needed.
- [ ] **Step 2:** Implement touch + evaluate commands.
- [ ] **Step 3:** Manual script in plan notes: set 5s timeout in debug, verify lock.
- [ ] **Step 4:** Commit `feat: wire idle auto-lock activity tracking`

### Task 1.2: History list / restore Tauri API

**Files:**
- Modify: `crates/onq-core/src/history.rs` — add `read_snapshot(path) -> String`, `restore(vault, id, snapshot_path) -> ()` (writes body via vault save path)
- Modify: `crates/onq-app/src/commands.rs` — `list_prompt_history(id)`, `restore_prompt_history(id, snapshot_path)`
- Test: extend `crates/onq-core/tests/integration/history.rs`

**API shape (canonical):**

```rust
// list_prompt_history -> Vec<{ path: String, timestamp: String, bytes: u64 }>
// restore_prompt_history(id, path) -> PromptDetail  // reindexes embedding best-effort
```

- [ ] **Step 1:** Failing integration test: edit twice → two snapshots → restore older → body matches.
- [ ] **Step 2:** Implement core + commands.
- [ ] **Step 3:** `cargo test -p onq-core --test history`
- [ ] **Step 4:** Commit `feat: history list and restore commands`

### Task 1.3: History UI in Editor

**Files:**
- Create: `src/lib/api/history.ts`
- Create: `src/lib/components/HistoryPanel.svelte` (or section in Editor)
- Modify: `src/lib/components/Editor.svelte` — “History” disclosure; pick snapshot; show DiffViewer vs current; Restore confirm
- Reuse: `DiffViewer.svelte`

- [ ] **Step 1:** Vitest with mocked list/restore.
- [ ] **Step 2:** UI implementation.
- [ ] **Step 3:** Commit `feat: prompt history browser in editor`

### Task 1.4: Configurable history retention

**Files:**
- Modify: `crates/onq-core/src/vault.rs` — `prune_older_than(self, days)` uses setting-provided days (passed from app layer)
- Modify: save paths in commands to pass `history_retention_days`
- Settings already from Task 0.3

- [ ] **Step 1:** Test prune with days=0 removes all / days=365 keeps.
- [ ] **Step 2:** Wire setting.
- [ ] **Step 3:** Commit `feat: configurable history retention days`

**Wave 1 verify:**  
`cargo test -p onq-core --test history && cargo test -p onQ && npm test -- --run`

---

# Wave 2 — Search & organization completeness

**Depends on:** Wave 0 settings patterns.

### Task 2.1: Tag suggest end-to-end

**Files:**
- Modify: `crates/onq-app/src/commands.rs` — `suggest_tags_for_body(body: String, max_n: usize) -> Vec<String>`  
  Vocabulary = union of all tags from prompts (query index or scan vault frontmatter).
- Create: `src/lib/api/tags.ts`
- Modify: `src/lib/components/Editor.svelte` — chips under tags field; Tab accepts first; Esc dismisses
- Test: Rust unit already in `tag_suggest.rs`; add command test with fixture vault; Vitest for key handling

- [ ] **Step 1:** Command returns ranked tags for known body against vault vocab.
- [ ] **Step 2:** Editor UX.
- [ ] **Step 3:** Commit `feat: tag suggestions in editor`
- [ ] **Step 4:** Update docs to match real behavior (still vocab-based, not zero-shot).

### Task 2.2: Visual smart-folder builder

**Files:**
- Create: `src/lib/smartFolders/visualModel.ts` — types for predicates `{ field, op, value }[]` + combine
- Create: `src/lib/smartFolders/visualCodec.ts` — `visualToDsl` / `dslToVisual` (best-effort; unsupported DSL stays text-only)
- Modify: `src/lib/api/smartFolders.ts` — pass `queryVisual` on create/update
- Modify: `crates/onq-app/src/commands.rs` — `create_smart_folder` / `update_smart_folder` accept optional visual JSON (already has column)
- Modify: `src/lib/components/LibraryPage.svelte` — toggle Visual | DSL; chip UI for tag/folder/favorite/locked/recent

**Codec rules:**
- Visual → DSL must round-trip through `smart_folder_dsl::parse` without loss for supported fields.
- Unknown DSL tokens: show “Advanced DSL” and disable visual editor until cleaned.

- [ ] **Step 1:** Unit tests for codec round-trip (Vitest + Rust parse).
- [ ] **Step 2:** Persist visual JSON.
- [ ] **Step 3:** UI.
- [ ] **Step 4:** Commit `feat: visual smart folder builder`

### Task 2.3: Populate body_sparse / body_minhash on write

**Files:**
- Modify: `crates/onq-core/src/search.rs` / indexing write path in `commands.rs` / vault index update — when embedding/indexing a prompt, write sparse encoding to `PROMPTS_BODY_SPARSE` and minhash to `PROMPTS_BODY_MINHASH` per MongrelDB column contracts
- Modify: `crates/onq-core/src/schema.rs` comments once writers exist
- Test: integration index write reads non-empty auxiliary columns

- [ ] **Step 1:** Read MongrelDB index requirements for Sparse/MinHash columns in current core version; implement encode helpers.
- [ ] **Step 2:** Hook create/save/reindex paths.
- [ ] **Step 3:** `cargo test -p onq-core` indexing tests.
- [ ] **Step 4:** Commit `feat: write sparse and minhash body columns`

### Task 2.4: Search hit highlight polish

**Files:**
- Modify: search result DTO if needed (`snippet`, `highlight_ranges`)
- Modify: `src/lib/components/Palette.svelte` / library result rows — render highlighted snippets
- Test: unit for highlight range computation

- [ ] **Step 1:** Define snippet algorithm (first match window ±N chars).
- [ ] **Step 2:** UI render with `<mark>`.
- [ ] **Step 3:** Commit `feat: search result snippets with highlights`

**Wave 2 verify:**  
`cargo test -p onq-core && cargo test -p onQ && npm test -- --run`

---

# Wave 3 — Daily launcher power

**Depends on:** Wave 0–1 (copy path stable; lock interactions understood).

### Task 3.1: Template variable parser

**Files:**
- Create: `crates/onq-core/src/template.rs` — parse `{{name}}`, `{{name|default}}`, escape `\{\{`
- Create: `crates/onq-core/src/template.rs` tests
- Export from `lib.rs`

**API:**

```rust
pub struct TemplateField { pub name: String, pub default: Option<String> }
pub fn parse_template(body: &str) -> Vec<TemplateField>; // unique order of appearance
pub fn render_template(body: &str, values: &HashMap<String, String>) -> String;
```

- [ ] **Step 1:** TDD parser/render edge cases (nested braces invalid, defaults, missing → empty or keep placeholder — **decision: missing uses default else empty string**).
- [ ] **Step 2:** Commit `feat: prompt template parse and render`

### Task 3.2: Fill-on-copy UI

**Files:**
- Create: `src/lib/components/TemplateFillDialog.svelte`
- Modify: `src/lib/components/Palette.svelte` — before clipboard write, if fields non-empty open dialog
- Create: `src/lib/api/template.ts` or parse in TS via command `parse_template` / `render_template` (prefer Rust for one source of truth)
- Commands: `preview_template(body)`, `render_template(body, values: HashMap)`

- [ ] **Step 1:** Vitest dialog + mocked parse.
- [ ] **Step 2:** Wire palette + editor “Copy with fill”.
- [ ] **Step 3:** Commit `feat: interactive template fill on copy`

### Task 3.3: Multi-vault recent list + switcher

**Files:**
- Modify: app_state / settings key `recent_vaults` JSON array of paths (max 10)
- Modify: `open_last_vault` / unlock / setup to push path onto recent list
- Modify: `src/lib/components/EmptyState.svelte` / `VaultUnlock.svelte` / Settings Vault — list recent, open other, remove entry
- Modify: `crates/onq-app/src/state.rs` — switch = `close_vault` then open/unlock new path
- Command: `list_recent_vaults`, `switch_vault(path, password?)`, `remove_recent_vault(path)`

- [ ] **Step 1:** Rust tests for recent list ring buffer.
- [ ] **Step 2:** UI switcher.
- [ ] **Step 3:** Commit `feat: multi-vault recent list and switcher`

**Wave 3 verify:**  
`cargo test -p onq-core template && cargo test -p onQ && npm test -- --run`

---

# Wave 4 — Migration & portability

**Depends on:** Wave 3 multi-vault helpful but not required.

### Task 4.1: Bulk import pipeline

**Files:**
- Create: `crates/onq-core/src/import/mod.rs` — `ImportReport { created, skipped, errors }`
- Create: `crates/onq-core/src/import/markdown_dir.rs` — walk `.md`, parse frontmatter, assign ids if missing
- Create: `crates/onq-core/src/import/json_array.rs` — `[{title,body,tags,folder}]`
- Create: `crates/onq-core/src/import/chatgpt_export.rs` — best-effort conversations.json / export zip heuristics
- Commands: `import_prompts(path, format: auto|markdown|json|chatgpt)`
- UI: Settings → Vault → Import… (file/dir dialog)

**Rules:**
- Never overwrite existing id without confirm flag `on_conflict: skip|replace`.
- Reindex embeddings best-effort after batch (progress events optional).

- [ ] **Step 1:** Integration tests with fixture dirs under `tests/fixtures/import/`.
- [ ] **Step 2:** Commands + UI.
- [ ] **Step 3:** Commit `feat: bulk prompt import`

### Task 4.2: Curated export

**Files:**
- Create: `crates/onq-core/src/export.rs` — filter by tags/folders/favorites; write folder or zip of `.md`
- Command: `export_prompts(filter, dest)`
- UI: Library multi-select + “Export selected…”; Settings full export (already have vault folder + backup)

- [ ] **Step 1:** Test export filter includes only matching tags.
- [ ] **Step 2:** UI.
- [ ] **Step 3:** Commit `feat: curated prompt export`

### Task 4.3: Backup reminders (local only)

**Files:**
- Settings: `backup_remind_days` (0 = off, default 7)
- On launch: if last backup timestamp older than N days, soft banner in Library/Settings
- Record last backup time when export_vault_backup succeeds (`last_backup_at` app setting)

- [ ] **Step 1:** Unit test for “should remind” pure function.
- [ ] **Step 2:** Banner UI.
- [ ] **Step 3:** Commit `feat: local backup age reminders`

**Wave 4 verify:**  
Import fixture round-trip + export filter tests + `npm test -- --run`

---

# Wave 5 — Extensibility & editor depth

**Depends on:** Wave 0 honesty; benefits from Wave 2 search.

### Task 5.1: Plugin manager UI

**Files:**
- Create: `src/lib/api/plugins.ts` — wrap install/list/enable/uninstall
- Create: `src/lib/components/settings/PluginsSection.svelte`
- Modify: `SettingsPage.svelte` — new section `plugins`
- UX: pick `.onqplugin` / archive file; show id, version, signature status, enable toggle, uninstall confirm
- Errors: unsigned/quarantined shown clearly

- [ ] **Step 1:** Vitest with mocked plugin list.
- [ ] **Step 2:** UI + wire commands.
- [ ] **Step 3:** Commit `feat: plugin manager in settings`

### Task 5.2: Plugin authoring guide + sample plugin refresh

**Files:**
- Create: `docs/plugins/README.md` — ABI, signing, capabilities, load path, example
- Modify: `tests/fixtures/sample-plugin/` if needed to match ABI 1
- Link from `docs-site/usage.md` and README

- [ ] **Step 1:** Write guide with copy-paste manifest + `export_plugin!` example.
- [ ] **Step 2:** Build sample plugin in CI optional job or documented `cargo build`.
- [ ] **Step 3:** Commit `docs: plugin authoring guide`

### Task 5.3: Palette integration for plugin commands

**Files:**
- Modify: plugin host to collect `register_command` registrations into `AppState`
- Command: `list_plugin_commands` → palette section “Plugin commands”
- Test: sample plugin registers a no-op command visible in list

- [ ] **Step 1:** Host-side registry.
- [ ] **Step 2:** Palette UI.
- [ ] **Step 3:** Commit `feat: show plugin commands in palette`

### Task 5.4: Alternate embedder via plugin (hook only)

**Files:**
- Modify: `crates/onq-core/src/embed.rs` / app embedder selection — trait object or enum `Builtin | Plugin`
- Settings: if plugin advertises `embedding` capability, allow select; default remains MiniLM
- **Do not** download arbitrary models in core without user action

- [ ] **Step 1:** Design embedder trait; MiniLM implements it.
- [ ] **Step 2:** Plugin path calls `HostApi.embedding_embed` reverse or plugin-provided symbol — document chosen approach in ADR under `docs/adr/0001-plugin-embedders.md`.
- [ ] **Step 3:** Commit `feat: pluggable embedder selection hook`

### Task 5.5: Editor Markdown preview

**Files:**
- Modify: `src/lib/components/Editor.svelte` — Edit | Preview | Split
- Dependency: prefer existing stack; if markdown lib needed, evaluate `marked` or `markdown-it` — **slopcheck before add**; sanitize HTML
- Test: Vitest render known markdown → expected elements

- [ ] **Step 1:** Add dep only if necessary; otherwise lightweight custom subset.
- [ ] **Step 2:** Preview pane.
- [ ] **Step 3:** Commit `feat: markdown preview in editor`

**Wave 5 verify:**  
Plugin install test + docs links + `npm test -- --run` + `cargo test -p onq-core`

---

# Wave 6 — 1.0 polish & stretch

**Depends on:** Waves 0–5 for 1.0 candidate; stretch items may slip to 1.x.

### Task 6.1: Security audit log (local)

**Files:**
- Create: `crates/onq-core/src/audit.rs` — append-only JSONL under `.onq/audit.log` (or encrypted sibling)
- Events: vault_unlock, vault_lock, export_backup, import_backup, prompt_unlock, plugin_install
- Settings: enable/disable; view last N in Settings → Vault (read-only)

- [ ] **Step 1:** Unit test append + read.
- [ ] **Step 2:** Instrument commands.
- [ ] **Step 3:** Commit `feat: local security audit log`

### Task 6.2: i18n scaffold

**Files:**
- Create: `src/lib/i18n/en.ts` message catalog
- Wire critical strings (palette placeholder, settings labels, errors) through `t(key)`
- **Only English catalog required for 1.0**; structure allows later locales

- [ ] **Step 1:** Introduce `t()` without changing copy meaning.
- [ ] **Step 2:** Migrate high-traffic strings.
- [ ] **Step 3:** Commit `feat: i18n message catalog scaffold`

### Task 6.3: Search quality / dense ANN UX polish

**Files:**
- Settings already has embedding quant + ensure model
- Add: progress UI for re-embed all; first-run soft prompt to download MiniLM
- Bench: optional `ann_rerank` variant in `search_bench.rs` when model present

- [ ] **Step 1:** First-run / empty-embed banner.
- [ ] **Step 2:** Re-embed progress events.
- [ ] **Step 3:** Commit `feat: embedding onboarding and reindex progress`

### Task 6.4: Explicit deferrals (document, do not build in 1.0)

Record in `docs/superpowers/plans/DEFERRED.md`:

| Item | Why deferred |
|------|----------------|
| Mobile apps | Separate native/Flutter effort |
| Browser extension “send to model” | Needs extension store + bridge protocol |
| Real zero-shot tag classifier | Optional plugin; vocab suggest is enough |
| Team multiplayer / comments | Conflicts with local-first core |
| Cloud sync service | Users already use Git/Dropbox/Syncthing |
| Full MongrelDB Kit migration | Large rewrite; no user-facing win in-band |

- [ ] **Step 1:** Write DEFERRED.md with revisit criteria.
- [ ] **Step 2:** Commit `docs: defer post-1.0 platform expansions`

### Task 6.5: 1.0 release gate

- [ ] Run full `cargo test --workspace`, `npm test -- --run`, Playwright a11y suite
- [ ] `cargo deny check` (or project equivalent)
- [ ] Manual UAT checklist: create vault, search hybrid, lock prompt, idle lock, history restore, template copy, import fixtures, plugin install, backup export/import, multi-vault switch
- [ ] Version bump + release notes
- [ ] Tag `v1.0.0` only when pre-alpha disclaimer is removed from README with intentional decision

---

## Cross-cutting work (every wave)

| Concern | Practice |
|---------|----------|
| **Migrations** | Any schema change → `migrate.rs` + version bump; never break existing vaults |
| **Legal inventory** | New crates → refresh `crates/onq-app/legal/crates.json` + third-party.md via existing scripts |
| **A11y** | New UI: keyboard path + aria labels; extend Playwright a11y where flows change |
| **Security** | Lock/unlock, import, plugin install: treat as `security: high` in review |
| **Perf** | Import/reindex batches use `spawn_blocking`; no UI jank on 1k prompts |
| **Commits** | One logical feature per commit; no AI attribution |

---

## Suggested schedule (single engineer, order-of-magnitude)

| Wave | Calendar (full-time focus) | Notes |
|------|----------------------------|-------|
| 0 | 3–5 days | Foundation |
| 1 | 1–1.5 weeks | Auto-lock + history |
| 2 | 1.5–2 weeks | Suggest, visual SF, index columns |
| 3 | 1–1.5 weeks | Templates + multi-vault |
| 4 | 1.5–2 weeks | Import/export (ChatGPT heuristics flaky — time-box) |
| 5 | 2 weeks | Plugins + preview |
| 6 | 1–2 weeks | Polish + gate |
| **Total** | **~8–12 weeks** | Parallel pair can cut ~30–40% |

---

## Execution model

1. **One wave = one (or few) feature branches**, not one branch for all waves.
2. At the start of each wave, expand its tasks into a detailed TDD plan if agents implement (subagent-driven-development).
3. After each wave merge: run Wave verify commands; update this file’s checkboxes.
4. If a task is cut, move it to `DEFERRED.md` with reason — do not silently drop.

### Agent execution choice (per wave)

**1. Subagent-Driven (recommended)** — fresh subagent per task, review between tasks  
**2. Inline Execution** — same session with checkpoints  

---

## Spec coverage checklist (self-review)

| Gap from product review | Wave / Task |
|-------------------------|-------------|
| Docs over-claim (tag classifier, plugins TBD) | 0.1, 2.1 docs, 5.2 |
| Idle auto-lock no-op | 1.1 |
| Lock vault now | 0.2 |
| History UI / restore | 1.2–1.4 |
| Tag suggest UI | 2.1 |
| Visual smart folders | 2.2 |
| Recency setting | 0.3–0.4 |
| body_sparse / minhash writers | 2.3 |
| Search snippets | 2.4 |
| Template variables | 3.1–3.2 |
| Multi-vault switcher | 3.3 |
| Bulk import | 4.1 |
| Curated export | 4.2 |
| Backup reminders | 4.3 |
| Plugin manager UI | 5.1 |
| Plugin docs | 5.2 |
| Plugin palette commands | 5.3 |
| Alternate embedders | 5.4 |
| Markdown preview | 5.5 |
| Audit log | 6.1 |
| i18n | 6.2 |
| Embedding onboarding | 6.3 |
| Mobile / extension / team / cloud / Kit migration | 6.4 deferred |

---

## Out of scope for this plan’s implementation commits

- Migrating storage layer to `mongreldb-kit`
- Changing license or adding telemetry
- Rewriting UI framework away from Svelte 5
- Replacing MiniLM as the **default** without a settings escape hatch

---

## First concrete next step

When ready to execute:

```text
Wave 0 → Task 0.2 (lock_vault_now) in parallel with Task 0.1 (docs honesty)
then Task 0.3 + 0.4 (settings + recency parameterization)
```

Kickoff:

```bash
# from repo root, after choosing a branch name
git checkout -b feat/wave0-foundation
```

Then implement Task 0.2 with TDD as specified above.
