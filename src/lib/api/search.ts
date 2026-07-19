import { invoke } from '@tauri-apps/api/core';
import type { SearchHit, SearchQuery } from '$lib/types/search';

/**
 * Run a hybrid search against the open vault's encrypted search index.
 *
 * The Rust side wraps mongreldb calls in `spawn_blocking` so the async
 * runtime isn't stalled. The frontend returns immediately and the search
 * itself runs on a worker thread.
 *
 * Returns an empty array when the embedder hasn't been loaded yet — the
 * backend degrades gracefully instead of erroring.
 */
export async function search(query: SearchQuery): Promise<SearchHit[]> {
  return invoke<SearchHit[]>('search', { query });
}

/**
 * MinHash "more like this": find the top `k` prompts whose stored set has the
 * highest estimated Jaccard similarity with the shingled body of `prompt_id`.
 *
 * The backend reshingles the source prompt's body into character trigrams on
 * the fly, so the command works even before any body-derived index column has
 * been repopulated for the source. The source prompt itself is always
 * dropped from the returned hit list.
 *
 * The `rrf_score` field on each `SearchHit` carries the estimated Jaccard
 * (a `f32` widened to `f64`) — directly comparable across calls.
 *
 * Errors:
 * - `"vault not unlocked"` — the DB handle isn't loaded in `AppState.db`.
 * - `"source prompt not found"` — the id resolved no row in the search index.
 */
export async function moreLikeThis(
  promptId: string,
  k: number,
): Promise<SearchHit[]> {
  return invoke<SearchHit[]>('more_like_this', { promptId, k });
}