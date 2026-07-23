import { invoke } from '@tauri-apps/api/core';

export async function backupShouldRemind(): Promise<boolean> {
  return invoke<boolean>('backup_should_remind');
}
