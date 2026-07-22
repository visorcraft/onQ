import { test, expect, type AxeViolation } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { installTauriMock, openPalette } from './tauri-mock';

/**
 * WCAG 2.1 AA accessibility audit using axe-core.
 *
 * Strategy: run axe against every meaningful app surface reachable without
 * the Tauri backend (home, palette dialog) and log violations. The home-page
 * assertion is strict (zero violations); the palette dialog asserts only
 * critical/serious (the baseline shipped in M6.6 — minor issues can be
 * ratcheted down in follow-up tasks). Editor + tutorial surfaces require a
 * running Tauri backend so they aren't covered here yet.
 */

const TAGS = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'best-practice'];

function violationSummary(v: AxeViolation) {
  return {
    id: v.id,
    impact: v.impact ?? 'unknown',
    help: v.help,
    helpUrl: v.helpUrl,
    nodes: v.nodes.length,
    sample: v.nodes[0]?.target,
  };
}

function logViolations(label: string, violations: AxeViolation[]) {
  if (violations.length === 0) {
    console.log(`[a11y] ${label}: 0 violations`);
    return;
  }
  console.log(
    `[a11y] ${label}: ${violations.length} violation(s):`,
    JSON.stringify(violations.map(violationSummary), null, 2),
  );
}

test.describe('WCAG 2.1 AA — axe-core', () => {
  test('home page has no axe violations', async ({ page }) => {
    // Empty-state home (no vault) is the first-run surface.
    await installTauriMock(page, { vaultOpen: false });
    await page.goto('/');
    await expect(page.getByRole('heading', { name: 'onQ' })).toBeVisible();

    const results = await new AxeBuilder({ page }).withTags(TAGS).analyze();
    logViolations('home', results.violations);

    expect(results.violations).toEqual([]);
  });

  test('palette dialog has no critical/serious axe violations', async ({ page }) => {
    await installTauriMock(page, { vaultOpen: true });
    await page.goto('/');
    await expect(page.getByRole('img', { name: 'onQ' })).toBeVisible();
    await openPalette(page);
    const palette = page.getByRole('dialog', { name: 'Command palette' });
    await expect(palette).toBeVisible();

    const results = await new AxeBuilder({ page })
      .withTags(TAGS)
      .include('[role="dialog"][aria-label="Command palette"]')
      .analyze();
    logViolations('palette', results.violations);

    const blocking = results.violations.filter((v) => v.impact !== 'minor');
    expect(
      blocking,
      `blocking palette violations: ${JSON.stringify(blocking.map(violationSummary))}`,
    ).toEqual([]);
  });
});
