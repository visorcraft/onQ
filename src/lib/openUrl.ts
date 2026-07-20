/**
 * Open an https/http URL in the system browser.
 *
 * Tauri webviews block `window.open`; use the opener plugin instead.
 * Falls back to `window.open` only outside the Tauri runtime (tests / browser).
 */
export async function openExternalUrl(url: string): Promise<void> {
  if (!url) return;
  try {
    const { openUrl } = await import('@tauri-apps/plugin-opener');
    await openUrl(url);
  } catch {
    // Dev in plain browser / unit tests without the plugin bridge.
    window.open(url, '_blank', 'noopener,noreferrer');
  }
}
