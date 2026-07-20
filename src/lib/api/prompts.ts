import { invoke } from '@tauri-apps/api/core';
import type { PromptSummary } from '../types/prompt';

export async function openVault(path: string): Promise<void> {
  return invoke('open_vault', { path });
}
export async function listPrompts(): Promise<PromptSummary[]> {
  return invoke<PromptSummary[]>('list_prompts');
}
export async function readPrompt(id: string): Promise<PromptSummary> {
  return invoke<PromptSummary>('read_prompt', { id });
}
export async function createPrompt(title: string): Promise<PromptSummary> {
  return invoke<PromptSummary>('create_prompt', { title });
}
export async function savePrompt(args: {
  id: string;
  title: string;
  body: string;
  folder: string | null;
  tags: string[];
  favorite: boolean;
}): Promise<PromptSummary> {
  return invoke<PromptSummary>('save_prompt', args);
}
export async function setPromptFavorite(
  id: string,
  favorite: boolean,
): Promise<PromptSummary> {
  return invoke<PromptSummary>('set_prompt_favorite', { id, favorite });
}
export async function deletePrompt(id: string): Promise<void> {
  return invoke('delete_prompt', { id });
}
