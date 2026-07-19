import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

import { results, runSearch } from './search';

describe('search store', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    invokeMock.mockReset();
    results.set([]);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('runSearch populates store with backend results', async () => {
    invokeMock.mockResolvedValueOnce([
      {
        id: '1',
        title: 'Match',
        folder: null,
        tags: ['a'],
        favorite: false,
        locked: false,
        char_count: 10,
        updated_at: 0,
        rrf_score: 0.5,
      },
    ]);
    runSearch({ text: 'match', limit: 10 });
    // Debounce is 150 ms — advance fake timers.
    await vi.advanceTimersByTimeAsync(200);
    expect(get(results)).toHaveLength(1);
    expect(get(results)[0].title).toBe('Match');
    expect(invokeMock).toHaveBeenCalledWith('search', {
      query: { text: 'match', limit: 10 },
    });
    // Successful searches must append the query to the user's recent-
    // searches column so the palette's "Recent" group stays current.
    await vi.waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith('record_search', {
        query: 'match',
      }),
    );
  });

  it('runSearch keeps previous results on error', async () => {
    invokeMock.mockResolvedValueOnce([
      {
        id: '1',
        title: 'Keep',
        folder: null,
        tags: [],
        favorite: false,
        locked: false,
        char_count: 0,
        updated_at: 0,
        rrf_score: 0.1,
      },
    ]);
    runSearch({ text: 'keep' });
    await vi.advanceTimersByTimeAsync(200);
    expect(get(results)).toHaveLength(1);

    invokeMock.mockRejectedValueOnce(new Error('embedder not loaded'));
    runSearch({ text: 'oops' });
    await vi.advanceTimersByTimeAsync(200);
    // Previous results remain visible rather than crashing the palette.
    expect(get(results)).toHaveLength(1);
    expect(get(results)[0].title).toBe('Keep');
  });
});