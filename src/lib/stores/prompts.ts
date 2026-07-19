import { writable } from 'svelte/store';
import * as api from '../api/prompts';
import type { PromptSummary } from '../types/prompt';

export const prompts = writable<PromptSummary[]>([]);

export async function loadPrompts(): Promise<void> {
  prompts.set(await api.listPrompts());
}

export async function createPrompt(title: string): Promise<PromptSummary> {
  const p = await api.createPrompt(title);
  prompts.update((list) => [...list, p]);
  return p;
}