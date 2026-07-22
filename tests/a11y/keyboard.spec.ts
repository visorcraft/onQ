import { test, expect } from '@playwright/test';
import { installTauriMock, openPalette } from './tauri-mock';

/**
 * Keyboard navigation smoke test.
 *
 * Verifies the basics of keyboard-only operation:
 *   1. Tab moves focus through the app (no focus traps).
 *   2. The currently-focused element shows a visible focus ring (the
 *      `:focus-visible` outline defined in App.svelte).
 *   3. The palette shortcut opens the command palette from anywhere.
 */

test('tabbing reveals a visible focus ring', async ({ page }) => {
  await installTauriMock(page, { vaultOpen: false });
  await page.goto('/');

  const outlines: string[] = [];
  for (let i = 0; i < 6; i++) {
    await page.keyboard.press('Tab');
    const outline = await page.evaluate(() => {
      const el = document.activeElement as HTMLElement | null;
      if (!el) return 'none';
      const cs = getComputedStyle(el);
      return `${cs.outlineStyle} ${cs.outlineWidth} ${cs.outlineColor}`;
    });
    outlines.push(outline);
  }

  const visible = outlines.some((o) => !o.startsWith('none'));
  expect(
    visible,
    `expected at least one tab stop to show a focus ring, got: ${JSON.stringify(outlines)}`,
  ).toBe(true);
});

test('default Q shortcut opens the command palette from the home page', async ({ page }) => {
  await installTauriMock(page, { vaultOpen: true });
  await page.goto('/');
  await expect(page.getByRole('img', { name: 'onQ' })).toBeVisible();

  await openPalette(page);

  await expect(page.getByRole('dialog', { name: 'Command palette' })).toBeVisible();
  await expect(page.getByPlaceholder('Search prompts, or type to create…')).toBeVisible();

  await page.keyboard.press('ArrowDown');
  await expect(page.getByRole('button', { name: '+ New prompt' })).toBeFocused();
});
