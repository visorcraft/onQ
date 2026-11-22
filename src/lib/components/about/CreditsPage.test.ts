import { readFileSync } from 'node:fs';
import { resolve as resolvePath } from 'node:path';
import { expect, it, vi } from 'vitest';
import { mount, tick, unmount } from 'svelte';

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

import CreditsPage from './CreditsPage.svelte';
import { resolve, setLocale } from '$lib/i18n';
import type { CreditsData } from '$lib/api/about';

const fixture: CreditsData = {
  crates: [
    {
      name: 'serde',
      version: '1.0.0',
      license: 'MIT',
      repository: 'https://example.com/serde',
    },
    {
      name: 'tokio',
      version: '1.0.0',
      license: 'MIT OR Apache-2.0',
      repository: '',
    },
  ],
  packages: [
    {
      name: 'svelte',
      version: '5.0.0',
      license: 'MIT',
      repository: 'https://example.com/svelte',
      role: 'runtime UI',
    },
    {
      name: 'vite',
      version: '6.0.0',
      license: 'MIT',
      repository: '',
      role: 'build tooling',
    },
  ],
  runtime: [
    {
      name: 'WebKitGTK',
      notes: 'GTK shell',
      licenses: 'LGPL-2.1',
      spdxId: 'LGPL-2.1-only',
      projectUrl: 'https://webkitgtk.org',
    },
    {
      name: 'glibc',
      notes: 'C library',
      licenses: 'LGPL-2.1',
      spdxId: '',
      projectUrl: 'https://glibc.org',
    },
  ],
  crateCount: 2,
  packageCount: 2,
  runtimeCount: 2,
};

function mockCredits(data: CreditsData | Promise<CreditsData>) {
  invokeMock.mockReset();
  invokeMock.mockImplementation((cmd: string) => {
    if (cmd === 'credits_data_cmd') return Promise.resolve(data);
    return Promise.reject(new Error(`unexpected invoke: ${cmd}`));
  });
}

function mountPage() {
  document.body.replaceChildren();
  setLocale('en');
  const target = document.createElement('div');
  document.body.append(target);
  const component = mount(CreditsPage, { target });
  return { component, target };
}

async function waitForSummary(target: HTMLElement) {
  await vi.waitFor(() => {
    expect(target.querySelector('.sub')?.textContent?.trim()).toBe(
      '2 Cargo crates · 2 npm packages · 2 runtime components',
    );
  });
}

it('renders English chrome and interpolated a11y labels', async () => {
  mockCredits(fixture);
  const { component, target } = mountPage();

  await waitForSummary(target);

  expect(target.querySelector('h1')?.textContent?.trim()).toBe('Credits');
  expect(target.querySelector('.sub')?.textContent?.trim()).toBe(
    '2 Cargo crates · 2 npm packages · 2 runtime components',
  );

  const npmInput = target.querySelectorAll('input[type="search"]')[0];
  expect(npmInput?.getAttribute('placeholder')).toBe(
    'Filter by package name, role, or license…',
  );

  expect(target.querySelector('button[title="License text"]')).not.toBeNull();
  expect(
    target.querySelector(
      'button[aria-label="License text for WebKitGTK"]',
    ),
  ).not.toBeNull();
  expect(
    target.querySelector(
      'button[aria-label="Open project for WebKitGTK"]',
    ),
  ).not.toBeNull();
  expect(
    target.querySelector('button[aria-label="Open svelte"]'),
  ).not.toBeNull();

  // The package with an empty repository renders no repository button.
  expect(target.querySelector('button[aria-label="Open vite"]')).toBeNull();

  await unmount(component);
  setLocale('en');
});

it('shows the loading text until the inventory resolves', async () => {
  let resolveCredits!: (data: CreditsData) => void;
  const deferred = new Promise<CreditsData>((resolve) => {
    resolveCredits = resolve;
  });
  mockCredits(deferred);
  const { component, target } = mountPage();

  await vi.waitFor(() => {
    expect(target.querySelector('.sub')?.textContent).toContain(
      'Loading inventory…',
    );
  });

  resolveCredits(fixture);
  await waitForSummary(target);

  await unmount(component);
  setLocale('en');
});

it('re-renders every string when the locale changes', async () => {
  mockCredits(fixture);
  const { component, target } = mountPage();
  await waitForSummary(target);

  setLocale('de');
  await tick();

  expect(target.querySelector('h1')?.textContent?.trim()).toBe(
    resolve('de', 'about.credits.title'),
  );
  expect(target.querySelector('h1')?.textContent?.trim()).not.toBe('Credits');

  const expectedThs = [
    resolve('de', 'about.credits.col.package'),
    resolve('de', 'about.credits.col.version'),
    resolve('de', 'about.credits.col.role'),
    resolve('de', 'about.credits.col.license'),
    resolve('de', 'about.credits.col.crate'),
    resolve('de', 'about.credits.col.version'),
    resolve('de', 'about.credits.col.license'),
  ];
  const ths = Array.from(target.querySelectorAll('th')).filter(
    (th) => th.textContent?.trim(),
  );
  expect(ths).toHaveLength(expectedThs.length);
  ths.forEach((th, i) => {
    expect(th.textContent?.trim()).toBe(expectedThs[i]);
  });

  expect(target.querySelector('.credits-runtime h2')?.textContent?.trim()).toBe(
    resolve('de', 'about.credits.runtime.title'),
  );

  const inputs = target.querySelectorAll('input[type="search"]');
  expect(inputs[0]?.getAttribute('placeholder')).toBe(
    resolve('de', 'about.credits.npm.filterPlaceholder'),
  );
  expect(inputs[1]?.getAttribute('placeholder')).toBe(
    resolve('de', 'about.credits.crates.filterPlaceholder'),
  );

  const labels = Array.from(target.querySelectorAll('.table-section-label'));
  expect(labels[0]?.textContent?.trim()).toBe(
    resolve('de', 'about.credits.npm.title'),
  );
  expect(labels[1]?.textContent?.trim()).toBe(
    resolve('de', 'about.credits.crates.title'),
  );

  const licenseBtn = target.querySelector(
    'button[aria-label="' +
      resolve('de', 'about.credits.runtime.licenseTextFor', { name: 'WebKitGTK' }) +
      '"]',
  );
  const projectBtn = target.querySelector(
    'button[aria-label="' +
      resolve('de', 'about.credits.runtime.openProjectFor', { name: 'WebKitGTK' }) +
      '"]',
  );
  expect(licenseBtn).not.toBeNull();
  expect(projectBtn).not.toBeNull();

  const ariaLabels = Array.from(target.querySelectorAll('[aria-label]')).map(
    (el) => el.getAttribute('aria-label') ?? '',
  );
  expect(
    ariaLabels.some((label) => label.startsWith('License text for')),
  ).toBe(false);
  expect(
    ariaLabels.some((label) => label.startsWith('Open project for')),
  ).toBe(false);

  await unmount(component);
  setLocale('en');
});

it('shows the localized fallback license text and closes the dialog', async () => {
  mockCredits(fixture);
  const { component, target } = mountPage();
  await waitForSummary(target);

  const rows = Array.from(target.querySelectorAll('.runtime-row'));
  const emptySpdxRow = rows.find((row) => row.textContent?.includes('glibc'));
  expect(emptySpdxRow).toBeDefined();
  const licenseButton = emptySpdxRow?.querySelector('button');
  expect(licenseButton).not.toBeNull();

  licenseButton?.click();
  await tick();

  const dialog = target.querySelector('[role="dialog"]');
  expect(dialog?.getAttribute('aria-label')).toBe('glibc');
  expect(dialog?.querySelector('pre')?.textContent).toBe(
    resolve('en', 'about.credits.runtime.noLicenseText'),
  );

  const invokedCommands = invokeMock.mock.calls.map(([cmd]) => cmd);
  expect(invokedCommands).not.toContain('runtime_license_text_cmd');

  const closeButton = Array.from(dialog?.querySelectorAll('button') ?? []).find(
    (button) => button.textContent?.trim() === 'Close',
  );
  expect(closeButton).not.toBeUndefined();
  closeButton?.click();
  await vi.waitFor(() => {
    expect(target.querySelector('[role="dialog"]')).toBeNull();
  });

  await unmount(component);
  setLocale('en');
});

it('keeps the markup free of hardcoded English chrome strings', () => {
  const src = readFileSync(
    resolvePath(process.cwd(), 'src/lib/components/about/CreditsPage.svelte'),
    'utf8',
  );

  expect(src).not.toMatch(/\b(aria-label|title|placeholder)="[A-Za-z]/);
  expect(src).not.toContain('Loading inventory');
  expect(src).not.toContain('Runtime components');
  expect(src).not.toContain('License expression');
  expect(src).not.toContain('Filter by ');
});
