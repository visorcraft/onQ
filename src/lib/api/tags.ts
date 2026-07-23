import { invoke } from '@tauri-apps/api/core';

export async function suggestTagsForBody(
  body: string,
  maxN = 5,
): Promise<string[]> {
  return invoke<string[]>('suggest_tags_for_body', { body, maxN });
}
