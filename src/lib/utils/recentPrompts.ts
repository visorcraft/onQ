/** localStorage key for recently used prompt ids (most-recent first). */
const STORAGE_KEY = 'onq.recent_prompt_ids';

/** How many recent prompts the palette shows. */
export const RECENT_PROMPTS_CAP = 5;

function readIds(): string[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((x): x is string => typeof x === 'string' && x.length > 0);
  } catch {
    return [];
  }
}

function writeIds(ids: string[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(ids.slice(0, RECENT_PROMPTS_CAP)));
  } catch {
    // Quota / private mode — ignore; Recent falls back to updated order.
  }
}

/** Return most-recent-first prompt ids from local history. */
export function getRecentPromptIds(): string[] {
  return readIds().slice(0, RECENT_PROMPTS_CAP);
}

/**
 * Move `id` to the head of the recent list (deduped). Call after copy or
 * edit so the palette "Recent" group reflects what the user just used.
 */
export function pushRecentPromptId(id: string): void {
  const trimmed = id.trim();
  if (!trimmed) return;
  const next = [trimmed, ...readIds().filter((x) => x !== trimmed)];
  writeIds(next);
}
