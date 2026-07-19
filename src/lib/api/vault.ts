import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

/**
 * Show the native folder picker so the user can choose (or create) a
 * vault directory. Returns the absolute path, or `null` if cancelled.
 */
export async function pickVaultDir(): Promise<string | null> {
  const picked = await open({
    directory: true,
    multiple: false,
    title: 'Select onQ vault',
  });
  return typeof picked === 'string' ? picked : null;
}

export interface OpenVaultStatus {
  path: string | null;
  opened: boolean;
  needsPassword: boolean;
  needsRecovery: boolean;
}

/** Unlock the last vault used by this app, if one has been remembered. */
export async function openLastVault(): Promise<OpenVaultStatus> {
  return invoke<OpenVaultStatus>('open_last_vault');
}

export async function unlockVault(
  path: string,
  masterPassword: string | null = null,
): Promise<OpenVaultStatus> {
  return invoke<OpenVaultStatus>('unlock_vault', { path, masterPassword });
}

/** Restore a vault's generated encryption key from its recovery phrase. */
export async function recoverVault(path: string, recoveryPhrase: string): Promise<void> {
  await invoke('recover_vault', { path, recoveryPhrase });
}

export async function getVaultAuthMode(): Promise<'keychain' | 'password'> {
  return invoke<'keychain' | 'password'>('get_vault_auth_mode');
}

export async function retrieveVaultKey(recoveryPhrase: string): Promise<string> {
  return invoke<string>('retrieve_vault_key', { recoveryPhrase });
}

/** Create a password vault or a no-password vault with a recovery phrase. */
export async function setupNewVault(
  path: string,
  masterPassword: string | null,
): Promise<{ recoveryPhrase: string | null }> {
  return invoke<{ recoveryPhrase: string | null }>('setup_new_vault', {
    path,
    masterPassword,
  });
}
