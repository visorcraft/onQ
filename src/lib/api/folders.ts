import { invoke } from '@tauri-apps/api/core';

export type Folder = {
  id: string;
  name: string;
  created: number;
  updated: number;
};

export async function listFolders(): Promise<Folder[]> {
  return invoke<Folder[]>('list_folders');
}

export async function createFolder(name: string): Promise<Folder> {
  return invoke<Folder>('create_folder', { name });
}

export async function renameFolder(oldName: string, newName: string): Promise<Folder> {
  return invoke<Folder>('rename_folder', { oldName, newName });
}

export async function deleteFolder(name: string): Promise<void> {
  return invoke('delete_folder', { name });
}
