import { invoke } from '@tauri-apps/api/core';

/**
 * Append a search query to the user's recent-searches list (deduped, most-
 * recent-first, capped at 20). Called from the search store after every
 * successful search so the palette's "Recent" group stays current.
 *
 * Errors are intentionally swallowed: a failed record is harmless — the
 * next successful search will overwrite the slot. The Tauri side returns
 * errors only when the vault isn't unlocked.
 */
export async function recordSearch(query: string): Promise<void> {
  try {
    await invoke('record_search', { query });
  } catch {
    // Recent-search tracking is best-effort; never block the search path.
  }
}

/**
 * Record the id of the prompt the user just opened. Read back on the next
 * app start to pre-load the editor with the same prompt.
 */
export async function recordOpen(promptId: string): Promise<void> {
  try {
    await invoke('record_open', { promptId });
  } catch {
    // Same best-effort policy as `recordSearch`.
  }
}
