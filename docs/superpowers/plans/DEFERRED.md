# Deferred / partial items

Explicitly tracked so roadmap gaps are not silent drops.

## Post-1.0 platform expansions

| Item | Why deferred | Revisit when |
|------|----------------|--------------|
| Mobile apps | Separate native effort | Product prioritizes mobile |
| Browser extension “send to model” | Needs extension store + bridge protocol | After desktop 1.0 stable |
| Real zero-shot tag classifier | Vocab-based suggest is enough for 1.0 | Users request ML tags |
| Team multiplayer / comments | Conflicts with local-first core | Explicit collaboration product |
| Cloud sync service | Users already use Git/Dropbox/Syncthing | Never (unless identity changes) |
| Full MongrelDB Kit migration | Large rewrite; no user-facing win in-band | Schema pain exceeds hand-rolled layer |

## In-wave partials (shipped core, thinner UI)

| Item | What shipped | What’s deferred |
|------|----------------|-----------------|
| Visual smart-folder **chip UI** (Wave 2.2) | `smart_folder_visual` codec + `visual_to_dsl` / `dsl_to_visual` commands | Library chip/predicate builder UI (DSL editor remains primary) |
| Plugin palette command registry (Wave 5.3) | Plugin install/list/enable/uninstall UI; HostApi has `register_command` | `list_plugin_commands` host registry + palette section for plugin commands |
| Alternate embedder runtime swap (Wave 5.4) | ADR `docs/adr/0001-plugin-embedders.md`; MiniLM default | Selecting a plugin embedder in Settings at runtime |
| Audit log Settings viewer (Wave 6.1) | `audit.rs` append/read + instrumentation hooks on restore/import/export | Full Settings → Audit panel and auto-instrument of every unlock/lock/backup/plugin event |
| Multi-vault simultaneous open | Recent-vault list, switch (close + remember), remove | Two vaults open in one process |

## Shipped end-to-end (do not re-defer)

- Auto-lock policy **persisted** to app config file; idle evaluate + activity tracking
- History list/restore UI; tag suggestions; search **snippets** in palette with `<mark>`
- Templates fill-on-copy; bulk import/export; backup remind; plugin manager UI; MD preview
- Multi-vault **recent list + switch/unlock** on empty state
