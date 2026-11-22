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

  it('highlights the full entity when the query contains &', () => {
    expect(highlightSnippet('Tom & Jerry', '&')).toBe('Tom <mark>&amp;</mark> Jerry');
  });

  it('does not match inside a generated entity', () => {
    expect(highlightSnippet('Q&A research', 'amp')).toBe('Q&amp;A research');
    expect(highlightSnippet('an example here', 'amp')).toBe('an ex<mark>amp</mark>le here');
  });

  it('does not match entity names generated from other characters', () => {
    expect(highlightSnippet('a < b', 'lt')).toBe('a &lt; b');
  });

  it('highlights literal special-character queries', () => {
    expect(highlightSnippet('a < b', '<')).toBe('a <mark>&lt;</mark> b');
    expect(highlightSnippet('say "hi"', '"')).toBe('say <mark>&quot;</mark>hi<mark>&quot;</mark>');
    expect(highlightSnippet('say "hi"', '"hi"')).toBe('say <mark>&quot;hi&quot;</mark>');
  });

  it('treats regex metacharacters in the query as literals', () => {
    expect(highlightSnippet('f(x) * 2', '*')).toBe('f(x) <mark>*</mark> 2');
    expect(highlightSnippet('a.b', '.')).toBe('a<mark>.</mark>b');
  });

  it('highlights multiple case-insensitive matches preserving casing', () => {
    expect(highlightSnippet('Cat CAT cart', 'cat')).toBe('<mark>Cat</mark> <mark>CAT</mark> cart');
  });
});
