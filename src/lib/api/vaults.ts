import { invoke } from '@tauri-apps/api/core';

/** Recent vault absolute paths (app config; works before unlock). */
export async function listRecentVaults(): Promise<string[]> {
  return invoke<string[]>('list_recent_vaults');
}

export async function pushRecentVault(path: string): Promise<void> {
  return invoke('push_recent_vault', { path });
}

export async function removeRecentVault(path: string): Promise<void> {
  return invoke('remove_recent_vault', { path });
}

/**
 * Close the current session and remember `path` as the last vault.
 * Caller must then unlock that path (password/keychain).
 */
export async function switchVault(path: string): Promise<void> {
  return invoke('switch_vault', { path });
}
