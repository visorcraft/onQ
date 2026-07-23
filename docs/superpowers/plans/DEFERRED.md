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
| Arbitrary plugin handler ABI | Host records handler ptr; safe typed invoke needs stable callback signature | Document plugin handler ABI |

## Shipped end-to-end (gap-closure waves)

- Auto-lock policy **persisted** + **loaded at startup**; idle evaluate writes **vault_lock** audit
- History list/restore UI; tag suggestions; search **snippets** with `<mark>`
- Smart-folder **visual chip builder** + DSL codec
- Templates fill-on-copy; bulk import/export; backup remind
- Plugin manager UI; HostApi **register_command** + `onq_plugin_init` load; palette refresh via `list_plugin_commands`
- Embedder preference setting
- Audit **enable/disable** + panel; events: vault_unlock/lock, idle lock, export_backup, import_backup, prompt_unlock, plugin_install, history_restore, import/export prompts
- Multi-vault **recent list + switch/unlock** on empty state
- i18n scaffold; plugin authoring docs; embedder ADR
