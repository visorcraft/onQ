import { invoke } from '@tauri-apps/api/core';
import type { PromptDetail } from '../types/prompt';

export type HistoryEntry = {
  path: string;
  timestamp: string;
  bytes: number;
};

export async function listPromptHistory(id: string): Promise<HistoryEntry[]> {
  return invoke<HistoryEntry[]>('list_prompt_history', { id });
}

export async function restorePromptHistory(
  id: string,
  snapshotPath: string,
): Promise<PromptDetail> {
  return invoke<PromptDetail>('restore_prompt_history', {
    id,
    snapshotPath,
  });
}
