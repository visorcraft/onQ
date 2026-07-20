import type { Page } from '@playwright/test';

/**
 * Stub Tauri IPC so browser-only a11y runs can reach the vault-open
 * surfaces (palette, hero) without a native backend.
 */
export async function installTauriMock(page: Page, opts: { vaultOpen?: boolean } = {}) {
  const vaultOpen = opts.vaultOpen ?? true;
  await page.addInitScript((open) => {
    const invoke = async (cmd: string, args?: { key?: string }) => {
      switch (cmd) {
        case 'open_last_vault':
          return open
            ? {
                path: '/tmp/onq-a11y-vault',
                opened: true,
                needsPassword: false,
                needsRecovery: false,
              }
            : {
                path: null,
                opened: false,
                needsPassword: false,
                needsRecovery: false,
              };
        case 'get_app_setting':
          if (args?.key === 'tutorial_completed') return 'true';
          if (args?.key === 'last_opened_prompt') return '';
          if (args?.key === 'recent_searches') return '[]';
          return '';
        case 'list_prompts':
        case 'search':
          return [];
        case 'ping':
          return 'onQ v1.0.2';
        default:
          return null;
      }
    };

    // Tauri 2 core reads window.__TAURI_INTERNALS__.invoke.
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (window as any).__TAURI_INTERNALS__ = {
      invoke,
      transformCallback: () => 0,
      unregisterCallback: () => undefined,
      convertFileSrc: (path: string) => path,
    };
  }, vaultOpen);
}

/** Palette shortcut works with either Meta or Control; CI is Linux. */
export async function openPalette(page: Page) {
  await page.locator('body').press('Control+k');
}
