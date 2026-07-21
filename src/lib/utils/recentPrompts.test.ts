import { describe, it, expect, beforeEach } from 'vitest';
import {
  getRecentPromptIds,
  pushRecentPromptId,
  RECENT_PROMPTS_CAP,
} from './recentPrompts';

describe('recentPrompts', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('starts empty', () => {
    expect(getRecentPromptIds()).toEqual([]);
  });

  it('prepends and dedupes', () => {
    pushRecentPromptId('a');
    pushRecentPromptId('b');
    pushRecentPromptId('a');
    expect(getRecentPromptIds()).toEqual(['a', 'b']);
  });

  it('caps list length', () => {
    for (let i = 0; i < RECENT_PROMPTS_CAP + 5; i++) {
      pushRecentPromptId(`id-${i}`);
    }
    const ids = getRecentPromptIds();
    expect(ids).toHaveLength(RECENT_PROMPTS_CAP);
    expect(ids[0]).toBe(`id-${RECENT_PROMPTS_CAP + 4}`);
  });

  it('ignores empty ids', () => {
    pushRecentPromptId('');
    pushRecentPromptId('   ');
    expect(getRecentPromptIds()).toEqual([]);
  });
});
