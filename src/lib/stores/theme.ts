import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export type Theme = 'dark' | 'light';
export const theme = writable<Theme>('dark');

function isTheme(value: unknown): value is Theme {
  return value === 'dark' || value === 'light';
}

/** Paint `t` on the document and the store. Does not touch the search index. */
export function applyTheme(t: Theme): void {
  document.documentElement.classList.toggle('light', t === 'light');
  document.documentElement.classList.toggle('dark', t === 'dark');
  theme.set(t);
}

/**
 * Apply `t` and persist it via `set_app_setting`. Throws if the backend
 * write fails — the UI surfaces this so the user can retry instead of
 * silently believing the toggle succeeded.
 */
export async function setTheme(t: Theme): Promise<void> {
  applyTheme(t);
  await invoke('set_app_setting', { key: 'theme', value: t });
}

/**
 * Pull the persisted theme and paint it. Does not write. A read-only load
 * used to call `setTheme`, and that extra `app_state` put could poison the
 * search index before the first prompt save. Missing, empty, or unknown
 * values paint `'dark'` (the seeded default) without a write. A locked
 * vault or a failed read leaves the theme already on screen.
 */
export async function loadTheme(): Promise<void> {
  try {
    const stored = await invoke<unknown>('get_app_setting', { key: 'theme' });
    applyTheme(isTheme(stored) ? stored : 'dark');
  } catch {
    // Vault locked or backend unavailable — keep the default theme.
  }
}
