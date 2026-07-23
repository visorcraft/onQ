# Deferred post-1.0 items

Explicitly tracked so roadmap gaps are not silent drops.

## Post-1.0 platform expansions (Wave 6.4)

| Item | Why deferred | Revisit when |
|------|----------------|--------------|
| Mobile apps | Separate native effort | Product prioritizes mobile |
| Browser extension “send to model” | Needs extension store + bridge protocol | After desktop 1.0 stable |
| Real zero-shot tag classifier | Vocab-based suggest is enough for 1.0 | Users request ML tags |
| Team multiplayer / comments | Conflicts with local-first core | Explicit collaboration product |
| Cloud sync service | Users already use Git/Dropbox/Syncthing | Never (unless identity changes) |
| Full MongrelDB Kit migration | Large rewrite; no user-facing win in-band | Schema pain exceeds hand-rolled layer |
| Multi-vault simultaneous open | One session design; recent list + switch covers UX | Users need concurrent vaults |

## Shipped end-to-end (gap-closure waves)

- Auto-lock policy **persisted** + **loaded at startup**; idle evaluate + activity tracking
- History list/restore UI; tag suggestions; search **snippets** with `<mark>`
- Smart-folder **visual chip builder** + DSL codec
- Templates fill-on-copy; bulk import/export; backup remind
- Plugin manager UI; **plugin palette commands** registry; embedder preference setting
- Audit module + **Settings audit panel**; unlock/lock instrumentation
- Multi-vault **recent list + switch/unlock** on empty state
- i18n scaffold; plugin authoring docs; embedder ADR
