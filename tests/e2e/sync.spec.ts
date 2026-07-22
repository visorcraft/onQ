import { expect, test } from '@playwright/test';
import { installTauriMock, openPalette } from '../a11y/tauri-mock';

test('app loads and opens the command palette', async ({ page }) => {
  await installTauriMock(page, { vaultOpen: true });
  await page.goto('/');

  await expect(page.getByRole('img', { name: 'onQ' })).toBeVisible();

  await openPalette(page);

  await expect(page.getByRole('dialog', { name: 'Command palette' })).toBeVisible();
  await expect(page.getByPlaceholder('Search prompts, or type to create…')).toBeVisible();
});
