/** Escape HTML then wrap case-insensitive query matches in <mark>. */
export function highlightSnippet(snippet: string, q: string): string {
  const esc = (s: string) =>
    s
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  const safe = esc(snippet);
  const needle = q.trim();
  if (!needle) return safe;
  try {
    const re = new RegExp(`(${needle.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'ig');
    return safe.replace(re, '<mark>$1</mark>');
  } catch {
    return safe;
  }
}
