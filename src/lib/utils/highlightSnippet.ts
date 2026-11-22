/** Escape HTML then wrap case-insensitive query matches in <mark>. */
export function highlightSnippet(snippet: string, q: string): string {
  const esc = (s: string) =>
    s
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  const needle = q.trim();
  if (!needle) return esc(snippet);
  try {
    const re = new RegExp(`(${needle.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'ig');
    const parts = snippet.split(re);
    return parts
      .map((part, i) => (i % 2 === 1 ? `<mark>${esc(part)}</mark>` : esc(part)))
      .join('');
  } catch {
    return esc(snippet);
  }
}
