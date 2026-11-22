import { describe, expect, it } from 'vitest';
import type { PromptSummary } from '$lib/types/prompt';
import {
  applyTextFilterAndSort,
  filterVisiblePrompts,
} from './libraryFilter';

let counter = 0;
function makePrompt(overrides: Partial<PromptSummary> = {}): PromptSummary {
  counter += 1;
  return {
    id: `p-${counter}`,
    title: 'Untitled',
    folder: null,
    tags: [],
    favorite: false,
    locked: false,
    updated: '2026-01-01T00:00:00.000Z',
    char_count: 0,
    preview: '',
    ...overrides,
  };
}

describe('filterVisiblePrompts', () => {
  it('favorites returns only favorite prompts', () => {
    const fav = makePrompt({ id: 'fav', favorite: true });
    const other = makePrompt({ id: 'other', favorite: false });
    const result = filterVisiblePrompts([fav, other], { kind: 'favorites' }, '');
    expect(result.map((p) => p.id)).toEqual(['fav']);
  });

  it('unfiled treats whitespace-only and missing folders as unfiled', () => {
    const blank = makePrompt({ id: 'blank', folder: '   ' });
    const missing = makePrompt({ id: 'missing', folder: undefined as unknown as null });
    const filed = makePrompt({ id: 'filed', folder: 'Writing' });
    const result = filterVisiblePrompts([blank, missing, filed], { kind: 'unfiled' }, '');
    expect(result.map((p) => p.id).sort()).toEqual(['blank', 'missing']);
  });

  it('tag matches whitespace-padded raw tags against a trimmed chip value', () => {
    const trailing = makePrompt({ id: 'trailing', tags: ['foo '] });
    const leading = makePrompt({ id: 'leading', tags: [' foo'] });
    const other = makePrompt({ id: 'other', tags: ['bar'] });
    const result = filterVisiblePrompts(
      [trailing, leading, other],
      { kind: 'tag', tag: 'foo' },
      '',
    );
    expect(result.map((p) => p.id).sort()).toEqual(['leading', 'trailing']);
  });

  it('tag is still case-sensitive', () => {
    const prompt = makePrompt({ id: 'cased', tags: ['Foo'] });
    expect(
      filterVisiblePrompts([prompt], { kind: 'tag', tag: 'foo' }, ''),
    ).toEqual([]);
    expect(
      filterVisiblePrompts([prompt], { kind: 'tag', tag: 'Foo' }, '').map((p) => p.id),
    ).toEqual(['cased']);
  });

  it('recent returns at most 30 items sorted by updated descending', () => {
    const prompts = Array.from({ length: 35 }, (_, i) =>
      makePrompt({
        id: `p-${i}`,
        updated: `2026-01-${String(i + 1).padStart(2, '0')}T00:00:00.000Z`,
      }),
    );
    const result = filterVisiblePrompts(prompts, { kind: 'recent' }, '');
    expect(result).toHaveLength(30);
    expect(result.map((p) => p.id).sort()).toEqual(
      Array.from({ length: 30 }, (_, i) => `p-${34 - i}`).sort(),
    );
    for (let i = 0; i < result.length - 1; i++) {
      expect(result[i].updated >= result[i + 1].updated).toBe(true);
    }
  });

  it('project only includes prompts whose trimmed folder is under the path', () => {
    const under = makePrompt({ id: 'under', folder: 'Writing/Blog' });
    const same = makePrompt({ id: 'same', folder: 'Writing' });
    const sibling = makePrompt({ id: 'sibling', folder: 'Coding' });
    const padded = makePrompt({ id: 'padded', folder: '  Writing/Research Notes  ' });
    const result = filterVisiblePrompts(
      [under, same, sibling, padded],
      { kind: 'project', path: 'Writing' },
      '',
    );
    expect(result.map((p) => p.id).sort()).toEqual(['padded', 'same', 'under']);
  });

  it('filterText matches title/preview/folder/tags case-insensitively and trims the query', () => {
    const byTitle = makePrompt({ id: 'title', title: 'Hello World' });
    const byPreview = makePrompt({ id: 'preview', title: 'x', preview: 'Some excerpt' });
    const byFolder = makePrompt({ id: 'folder', title: 'x', folder: 'Project Alpha' });
    const byTag = makePrompt({ id: 'tag', title: 'x', tags: ['Beta Tag'] });
    const noMatch = makePrompt({ id: 'none', title: 'x', preview: 'y', folder: 'z', tags: ['q'] });

    expect(
      filterVisiblePrompts([byTitle, noMatch], { kind: 'all' }, '  HELLO WORLD  ').map((p) => p.id),
    ).toEqual(['title']);
    expect(
      filterVisiblePrompts([byPreview, noMatch], { kind: 'all' }, 'excerpt').map((p) => p.id),
    ).toEqual(['preview']);
    expect(
      filterVisiblePrompts([byFolder, noMatch], { kind: 'all' }, 'project alpha').map((p) => p.id),
    ).toEqual(['folder']);
    expect(
      filterVisiblePrompts([byTag, noMatch], { kind: 'all' }, 'beta tag').map((p) => p.id),
    ).toEqual(['tag']);
  });

  it('sorts non-recent results by updated descending', () => {
    const older = makePrompt({ id: 'older', updated: '2026-01-01T00:00:00.000Z' });
    const newer = makePrompt({ id: 'newer', updated: '2026-06-01T00:00:00.000Z' });
    const result = filterVisiblePrompts([older, newer], { kind: 'all' }, '');
    expect(result.map((p) => p.id)).toEqual(['newer', 'older']);
  });

  it('handles empty tags/preview/folder without throwing', () => {
    const sparse = makePrompt({
      id: 'sparse',
      tags: undefined as unknown as string[],
      preview: undefined as unknown as string,
      folder: undefined as unknown as null,
    });
    expect(() => filterVisiblePrompts([sparse], { kind: 'all' }, '')).not.toThrow();
    expect(() => filterVisiblePrompts([sparse], { kind: 'favorites' }, '')).not.toThrow();
    expect(() => filterVisiblePrompts([sparse], { kind: 'recent' }, '')).not.toThrow();
    expect(() => filterVisiblePrompts([sparse], { kind: 'unfiled' }, '')).not.toThrow();
    expect(() =>
      filterVisiblePrompts([sparse], { kind: 'project', path: 'Writing' }, ''),
    ).not.toThrow();
    expect(() => filterVisiblePrompts([sparse], { kind: 'tag', tag: 'foo' }, '')).not.toThrow();
    expect(filterVisiblePrompts([sparse], { kind: 'all' }, 'anything')).toEqual([]);
  });
});

describe('applyTextFilterAndSort', () => {
  it('sorts by updated descending for non-recent selections', () => {
    const older = makePrompt({ id: 'older', updated: '2026-01-01T00:00:00.000Z' });
    const newer = makePrompt({ id: 'newer', updated: '2026-06-01T00:00:00.000Z' });
    const result = applyTextFilterAndSort([older, newer], '', false);
    expect(result.map((p) => p.id)).toEqual(['newer', 'older']);
  });

  it('keeps the given order for recent selections', () => {
    const older = makePrompt({ id: 'older', updated: '2026-01-01T00:00:00.000Z' });
    const newer = makePrompt({ id: 'newer', updated: '2026-06-01T00:00:00.000Z' });
    const result = applyTextFilterAndSort([older, newer], '', true);
    expect(result.map((p) => p.id)).toEqual(['older', 'newer']);
  });
});
