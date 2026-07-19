import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

import { prompts, loadPrompts, createPrompt } from './prompts';

describe('prompts store', () => {
  beforeEach(() => { invokeMock.mockReset(); });

  it('loadPrompts populates store', async () => {
    invokeMock.mockResolvedValueOnce([{ id: '1', title: 'A', tags: [], folder: null, favorite: false, locked: false, updated: '', char_count: 0 }]);
    await loadPrompts();
    expect(get(prompts)).toHaveLength(1);
    expect(get(prompts)[0].title).toBe('A');
  });

  it('createPrompt appends and returns new', async () => {
    invokeMock.mockResolvedValueOnce([]);
    await loadPrompts();
    invokeMock.mockResolvedValueOnce({ id: '2', title: 'New', tags: [], folder: null, favorite: false, locked: false, updated: '', char_count: 0 });
    const created = await createPrompt('New');
    expect(created.id).toBe('2');
    expect(get(prompts)).toHaveLength(1);
  });
});