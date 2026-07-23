/**
 * Settings → Backups API.
 *
 * Keeps dialog + invoke details out of the Settings view so future entry
 * points (palette commands, recovery flow) can reuse the same surface.
 */
import { open, save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

export interface BackupPaths {
  vaultPath: string;
  databasePath: string;
}

export interface ImportBackupResult {
  path: string;
  needsPassword: boolean;
}

const BACKUP_FILTER: { name: string; extensions: string[] } = {
  name: 'onQ backup',
  extensions: ['onqbak'],
};

/** Paths for the open vault (requires unlock). */
export async function getBackupPaths(): Promise<BackupPaths> {
  return invoke<BackupPaths>('get_backup_paths');
}

/** Probe whether a file is password-sealed without needing the password. */
export async function backupIsSealed(archivePath: string): Promise<boolean> {
  return invoke<boolean>('backup_is_sealed', { archivePath });
}

/**
 * Native save dialog for a new `.onqbak`, then export.
 * Returns the written path, or `null` if the user cancelled.
 */
export async function exportVaultBackup(password?: string | null): Promise<string | null> {
  const dest = await save({
    title: 'Export onQ vault backup',
    defaultPath: `onq-vault-${dateStamp()}.onqbak`,
    filters: [BACKUP_FILTER],
  });
  if (typeof dest !== 'string' || !dest) return null;
  const path = dest.endsWith('.onqbak') ? dest : `${dest}.onqbak`;
  await invoke<void>('export_vault_backup', {
    destPath: path,
    password: emptyToNull(password),
  });
  return path;
}

/**
 * Native open dialog for a `.onqbak`, then import over the open vault.
 * Returns `null` if cancelled. On success the vault session is closed.
 */
export async function importVaultBackup(
  password?: string | null,
): Promise<ImportBackupResult | null> {
  const picked = await open({
    title: 'Import onQ vault backup',
    multiple: false,
    filters: [BACKUP_FILTER],
  });
  if (typeof picked !== 'string' || !picked) return null;
  return invoke<ImportBackupResult>('import_vault_backup', {
    archivePath: picked,
    password: emptyToNull(password),
  });
}

/** Open-dialog only (so UI can probe seal mode before confirming import). */
export async function pickBackupArchive(): Promise<string | null> {
  const picked = await open({
    title: 'Import onQ vault backup',
    multiple: false,
    filters: [BACKUP_FILTER],
  });
  return typeof picked === 'string' ? picked : null;
}

/** Import a previously picked archive path. */
export async function importVaultBackupFromPath(
  archivePath: string,
  password?: string | null,
): Promise<ImportBackupResult> {
  return invoke<ImportBackupResult>('import_vault_backup', {
    archivePath,
    password: emptyToNull(password),
  });
}

function emptyToNull(password?: string | null): string | null {
  if (password == null) return null;
  const t = password.trim();
  return t.length === 0 ? null : t;
}

function dateStamp(): string {
  const d = new Date();
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}${m}${day}`;
}
