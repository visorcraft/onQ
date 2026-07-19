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
 * surface rejections as typed errors which the SettingsPanel rolls back.
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
 * - `dense`            — exact cosine brute-force scan over the filtered
 *   candidates. Slower than HNSW but functionally correct (true cosine
 *   similarity on full f32 vectors). The agreed fallback for the dense
 *   quantization mode — see `crates/onq-core/src/embedding_index.rs`.
 *
 * The `dense` value is recorded to `app_state.embedding_quant`
 * immediately and takes effect on the next search call; the on-disk
 * index stays binary until upstream MongrelDB exposes DROP/CREATE
 * INDEX DDL + a Dense AnnQuantization variant.
 */
export type EmbeddingQuant = 'binary' | 'dense';

export const embeddingQuant = writable<EmbeddingQuant>('binary');

/**
 * `true` while a `setEmbeddingQuant` call is in flight. The SettingsPanel
 * uses this to disable the radio buttons + show a progress affordance.
 */
export const rebuildingIndex = writable<boolean>(false);

/**
 * Switch the active embedding quantization. Persists to the backend,
 * which writes `app_state.embedding_quant` and triggers the
 * `set_embedding_quant` Tauri command (a no-op rebuild until upstream
 * MongrelDB gains the required DDL).
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
