import { describe, expect, it } from 'vitest';
import {
  applyPromptCounts,
  buildProjectTree,
  isUnder,
  leafName,
  normalizeProjectPath,
  parentPath,
  promptCountLabel,
} from './folderPath';

describe('folderPath', () => {
  it('normalizes paths', () => {
    expect(normalizeProjectPath('  Writing // Blog / ')).toBe('Writing/Blog');
  });

  it('builds a hierarchical tree', () => {
    const tree = buildProjectTree([
      'Writing/Blog Posts',
      'Writing/Research Notes',
      'Coding',
      'Writing',
    ]);
    expect(tree.map((n) => n.path)).toEqual(['Coding', 'Writing']);
    const writing = tree.find((n) => n.path === 'Writing')!;
    expect(writing.children.map((c) => c.name).sort()).toEqual([
      'Blog Posts',
      'Research Notes',
    ]);
  });

  it('counts descendants', () => {
    const tree = buildProjectTree(['Writing', 'Writing/Blog', 'Coding']);
    applyPromptCounts(tree, ['Writing/Blog', 'Writing/Blog', 'Coding', null]);
    expect(tree.find((n) => n.path === 'Writing')!.count).toBe(2);
    expect(tree.find((n) => n.path === 'Coding')!.count).toBe(1);
  });

  it('helpers', () => {
    expect(parentPath('a/b')).toBe('a');
    expect(leafName('a/b')).toBe('b');
    expect(isUnder('a/b', 'a')).toBe(true);
    expect(promptCountLabel(0)).toBe('0 prompts');
    expect(promptCountLabel(1)).toBe('1 prompt');
  });

  it('rejects control characters and overlong paths', () => {
    expect(() => normalizeProjectPath('bad\u0000name')).toThrow(/invalid/);
    expect(() => normalizeProjectPath('x'.repeat(201))).toThrow(/longer than/);
  });
});
