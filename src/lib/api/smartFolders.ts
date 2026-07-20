import { invoke } from '@tauri-apps/api/core';
import type { SearchHit } from '../types/search';

export type SmartFolder = {
  id: string;
  name: string;
  query_dsl: string;
  query_visual?: unknown;
  created?: number;
  updated?: number;
};

export async function listSmartFolders(): Promise<SmartFolder[]> {
  return invoke<SmartFolder[]>('list_smart_folders');
}

export async function createSmartFolder(
  name: string,
  queryDsl: string,
): Promise<SmartFolder> {
  return invoke<SmartFolder>('create_smart_folder', { name, queryDsl });
}

export async function updateSmartFolder(
  id: string,
  name: string,
  queryDsl: string,
): Promise<SmartFolder> {
  return invoke<SmartFolder>('update_smart_folder', { id, name, queryDsl });
}

export async function deleteSmartFolder(id: string): Promise<void> {
  return invoke('delete_smart_folder', { id });
}

export async function runSmartFolder(id: string): Promise<SearchHit[]> {
  return invoke<SearchHit[]>('run_smart_folder', { id });
}
