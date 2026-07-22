import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { getAppSetting, setAppSetting } from '$lib/api/settings';

/**
 * Serialise async writes through a single shared chain so two rapid
 * clicks on the same toggle can't race the round-trip to the backend.
 * Each `fn` runs after the previous one resolves (success OR failure);
 * the returned promise resolves with `fn`'s result. The chain swallows
 * its own errors so one rejection can't poison subsequent calls.
 */
function serialize<T>(fn: () => Promise<T>): Promise<T> {
  const next = pendingSettingsWrite.then(fn, fn);
  pendingSettingsWrite = next.catch(() => undefined);
  return next;
}

let pendingSettingsWrite: Promise<unknown> = Promise.resolve();

/**
 * User opt-in for pre-release auto-updates (M7.2).
 *
 * Persisted to `app_state.beta_channel`. Production updates use the stable
 * `updater` release feed (plus a GitHub-latest fallback). Splitting a
 * dedicated beta feed is intentionally a follow-up — toggling this today
 * only flips the stored flag. The UI surfaces that limitation inline.
 */
export const betaChannel = writable<boolean>(false);

/**
 * Persist the user's beta-channel opt-in. Returns the persisted value
 * so the caller doesn't have to re-read the store — Tauri commands
 * surface rejections as typed errors which the Settings page rolls back.
 */
export async function setBetaChannel(enabled: boolean): Promise<void> {
  await serialize(() => invoke<void>('set_beta_channel', { enabled }));
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
 * User opt-in to hiding the main window to the system tray immediately
 * after a palette copy (search result or "Recent" row). Persisted to
 * `app_state.minimize_on_copy` — defaults to `false` so the new vault
 * experience matches the shipped behaviour.
 */
export const minimizeOnCopy = writable<boolean>(false);

export async function setMinimizeOnCopy(enabled: boolean): Promise<void> {
  await serialize(() =>
    setAppSetting('minimize_on_copy', enabled ? 'true' : 'false'),
  );
  minimizeOnCopy.set(enabled);
}

export async function loadMinimizeOnCopy(): Promise<void> {
  try {
    const v = await getAppSetting('minimize_on_copy');
    minimizeOnCopy.set(v === 'true');
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
