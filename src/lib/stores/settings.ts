import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { getAppSetting } from '$lib/api/settings';

/**
 * User opt-in for pre-release auto-updates (M7.2).
 *
 * Persisted to `app_state.beta_channel`. The auto-update flow wired in
 * M7.1 still consults a single `latest.json` endpoint — splitting the
 * updater into separate production / beta channels is intentionally a
 * follow-up task, so toggling this today only flips the stored flag.
 * The UI surfaces that limitation inline so the user knows the choice
 * is recorded but does not yet redirect the updater.
 */
export const betaChannel = writable<boolean>(false);

/**
 * Persist the user's beta-channel opt-in. Returns the persisted value
 * so the caller doesn't have to re-read the store — Tauri commands
 * surface rejections as typed errors which the Settings page rolls back.
 */
export async function setBetaChannel(enabled: boolean): Promise<void> {
  await invoke<void>('set_beta_channel', { enabled });
  betaChannel.set(enabled);
}

/**
 * Read the persisted beta-channel flag from `app_state.beta_channel`.
 * Defaults to `false` when the column is unset or the vault is locked —
 * callers don't have to branch on first-run state.
 */
export async function loadBetaChannel(): Promise<void> {
  try {
    const v = await invoke<boolean>('get_beta_channel');
    betaChannel.set(Boolean(v));
  } catch {
    // Vault locked or backend unavailable — keep the default.
  }
}

/**
 * User-tunable embedding quantization mode for the `prompts.embedding`
 * Ann index.
 *
 * - `binary` (default) — binary HNSW candidate generation + exact cosine
 *   rerank. Fast; ~95% recall; low memory.
 * - `dense` — full-precision Dense ANN after replace-index publishes;
 *   exact cosine only while the rebuild is still pending. See
 *   `crates/onq-core/src/embedding_index.rs`.
 *
 * Preference is written to `app_state.embedding_quant` and a durable
 * replace-index job runs for `idx_prompts_embed_ann`.
 */
export type EmbeddingQuant = 'binary' | 'dense';

export const embeddingQuant = writable<EmbeddingQuant>('binary');

/**
 * `true` while a `setEmbeddingQuant` call is in flight. The Settings page
 * uses this to disable the radio buttons + show a progress affordance.
 */
export const rebuildingIndex = writable<boolean>(false);

/**
 * Switch the active embedding quantization. Persists preference and runs
 * the `set_embedding_quant` Tauri command (durable replace-index).
 *
 * Throws on a failed write — the UI catches and surfaces the error so
 * the user can retry instead of silently believing the toggle worked.
 */
export async function setEmbeddingQuant(q: EmbeddingQuant): Promise<void> {
  rebuildingIndex.set(true);
  try {
    await invoke<void>('set_embedding_quant', { quant: q });
    embeddingQuant.set(q);
  } finally {
    rebuildingIndex.set(false);
  }
}

/**
 * Read the persisted `embedding_quant` from the encrypted search-index
 * DB. Falls back to the seeded default (`'binary'`) when the vault is
 * locked or the column is unset — callers don't have to branch on
 * first-run state.
 */
export async function loadEmbeddingQuant(): Promise<void> {
  try {
    const v = await getAppSetting('embedding_quant');
    if (v === 'binary' || v === 'dense') {
      embeddingQuant.set(v);
    }
  } catch {
    // Vault locked or backend unavailable — keep the default.
  }
}
