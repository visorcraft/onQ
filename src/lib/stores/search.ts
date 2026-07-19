import { writable } from 'svelte/store';
import { search as searchApi, moreLikeThis as moreLikeThisApi } from '$lib/api/search';
import { recordSearch } from '$lib/api/recent';
import { debounce } from '$lib/utils/debounce';
import type { SearchHit, SearchQuery } from '$lib/types/search';

/** Latest hybrid-search results. Empty array until the first search completes. */
export const results = writable<SearchHit[]>([]);

/**
 * Debounced (150 ms) hybrid-search invocation. Wire this to the palette's
 * query input — every keystroke schedules a new search; only the latest
 * typed query actually hits the backend.
 *
 * The 150 ms debounce balances responsiveness against per-keystroke churn
 * on the encrypted mongreldb query path.
 */
export const runSearch = debounce(async (query: SearchQuery) => {
  try {
    const hits = await searchApi(query);
    results.set(hits);
    // Append successful queries to the recent-searches list so the
    // palette's "Recent" group stays current. `recordSearch` is fire-and-
    // forget on purpose (errors are swallowed) and runs in parallel with
    // the next search — by the time the user opens the palette again the
    // column is current.
    void recordSearch(query.text);
  } catch {
    // Search errors should never crash the palette — keep the previous
    // results on screen until the next successful query.
  }
}, 150);

/**
 * Run a MinHash "more like this" lookup and replace `$results` with the
 * returned hits. Used by the palette's `More like this: <title>` trigger
 * when an editor-bound prompt is selected.
 *
 * Errors are swallowed (same policy as `runSearch`); the user-visible
 * behaviour is "the palette continues to show whatever was there before".
 */
export async function runMoreLikeThis(promptId: string, k = 10): Promise<void> {
  try {
    results.set(await moreLikeThisApi(promptId, k));
  } catch {
    // MinHash errors should never crash the palette.
  }
}