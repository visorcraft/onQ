import { invoke } from '@tauri-apps/api/core';
import { writable } from 'svelte/store';

const LAST_STEP = 3;

export const tutorialStep = writable<number>(0);
export const tutorialVisible = writable<boolean>(false);

export async function startIfNeeded(): Promise<void> {
  const completed = await invoke<string>('get_app_setting', {
    key: 'tutorial_completed',
  });
  if (completed !== 'true') {
    tutorialVisible.set(true);
  }
}

export async function checkAndStart(vaultReady: boolean): Promise<void> {
  if (!vaultReady) return;
  await startIfNeeded();
}

export async function complete(): Promise<void> {
  tutorialVisible.set(false);
  try {
    await invoke('set_app_setting', {
      key: 'tutorial_completed',
      value: 'true',
    });
  } catch {
    // Persistence is best-effort; dismissal must never trap the user.
  }
}

export function next(): void {
  tutorialStep.update((step) => Math.min(step + 1, LAST_STEP));
}

export function prev(): void {
  tutorialStep.update((step) => Math.max(step - 1, 0));
}

export function reset(): void {
  tutorialStep.set(0);
  tutorialVisible.set(true);
}
