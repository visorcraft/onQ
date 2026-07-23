import { describe, expect, it } from 'vitest';
import { highlightSnippet } from './highlightSnippet';

describe('highlightSnippet', () => {
  it('marks case-insensitive matches and escapes HTML', () => {
    const html = highlightSnippet('Hello <World> TARGET here', 'target');
    expect(html).toContain('<mark>TARGET</mark>');
    expect(html).toContain('&lt;World&gt;');
    expect(html).not.toContain('<World>');
  });

  it('returns escaped text when query empty', () => {
    expect(highlightSnippet('a < b', '')).toBe('a &lt; b');
  });
});
