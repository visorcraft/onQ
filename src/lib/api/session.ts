import { invoke } from '@tauri-apps/api/core';

/** Close the open vault session; returns path for unlock UI (or ""). */
export async function lockVaultNow(): Promise<string> {
  return invoke<string>('lock_vault_now');
}

export async function touchActivity(): Promise<void> {
  return invoke('touch_activity');
}

/** Returns vault path when idle lock fired, otherwise null. */
export async function evaluateAutoLock(): Promise<string | null> {
  return invoke<string | null>('evaluate_auto_lock');
}

export async function getAutoLockPolicy(): Promise<string> {
  return invoke<string>('get_auto_lock_policy');
}

export async function setAutoLockPolicy(policy: string): Promise<void> {
  return invoke('set_auto_lock_policy', { policy });
}
