import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export type Theme = 'dark' | 'light';
export const theme = writable<Theme>('dark');

/**
 * Apply `t` to the DOM and persist it via the backend `set_app_setting`
 * command. Throws if the backend write fails — the UI surfaces this so the
 * user can retry instead of silently believing the toggle succeeded.
 */
export async function setTheme(t: Theme): Promise<void> {
  document.documentElement.classList.toggle('light', t === 'light');
  document.documentElement.classList.toggle('dark', t === 'dark');
  theme.set(t);
  await invoke('set_app_setting', { key: 'theme', value: t });
}

/**
 * Pull the persisted theme from the backend and apply it. Falls back to
 * `'dark'` (the seeded default) when the vault isn't unlocked yet — the
 * app_state row is only readable after unlock, so the first paint on a
 * locked vault is always dark.
 */
export async function loadTheme(): Promise<void> {
  try {
    const t = await invoke<Theme>('get_app_setting', { key: 'theme' });
    await setTheme(t);
  } catch {
    // Vault locked or backend unavailable — keep the default theme.
  }
}
