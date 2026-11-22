import { expect, it, vi } from 'vitest';
import { mount, tick, unmount } from 'svelte';

const { aboutInfoMock, openExternalUrlMock } = vi.hoisted(() => ({
  aboutInfoMock: vi.fn(),
  openExternalUrlMock: vi.fn(),
}));
vi.mock('$lib/api/about', () => ({ aboutInfo: aboutInfoMock }));
vi.mock('$lib/openUrl', () => ({ openExternalUrl: openExternalUrlMock }));

import AboutPage from './AboutPage.svelte';
import { resolve, setLocale } from '$lib/i18n';
import type { AboutInfo } from '$lib/api/about';

const fixture: AboutInfo = {
  appName: 'onQ',
  version: '1.14.4',
  license: 'GPL-3.0-only',
  repository: 'https://github.com/visorcraft/onQ',
  gitSha: 'abc123',
  description: 'Search-oriented encrypted prompt vault.',
  tagline: 'Built on Rust + Tauri 2 + Svelte 5.',
  platform: 'linux',
};

function mountPage() {
  document.body.replaceChildren();
  setLocale('en');
  const target = document.createElement('div');
  document.body.append(target);
  const component = mount(AboutPage, {
    target,
    props: {
      onLicenses: () => {},
      onCredits: () => {},
    },
  });
  return { component, target };
}

async function waitForLoaded(target: HTMLElement) {
  await vi.waitFor(() => {
    expect(target.querySelector('.about-hero h2')?.textContent?.trim()).toBe('onQ');
  });
}

function legalButtons(target: HTMLElement) {
  return Array.from(target.querySelectorAll('.about-legal-actions button')).map(
    (button) => button.textContent?.trim() ?? '',
  );
}

it('renders English chrome and both action buttons', async () => {
  aboutInfoMock.mockReset();
  aboutInfoMock.mockResolvedValue(fixture);
  openExternalUrlMock.mockReset();
  const { component, target } = mountPage();

  await waitForLoaded(target);

  const text = target.textContent ?? '';
  expect(text).toContain(resolve('en', 'about.feature.hybridSearch.title'));
  expect(text).toContain(resolve('en', 'about.whatsInside'));
  expect(text).toContain(resolve('en', 'about.repoCardCta'));
  expect(text).toContain(resolve('en', 'about.legalTitle'));
  expect(text).toContain(resolve('en', 'about.footer'));

  const buttons = legalButtons(target);
  expect(buttons.some((b) => b.includes(resolve('en', 'about.licenses')))).toBe(true);
  expect(buttons.some((b) => b.includes(resolve('en', 'about.credits.title')))).toBe(true);

  expect(target.querySelector('.about-link-card-url')?.textContent?.trim()).toBe(
    'github.com/visorcraft/onQ',
  );

  await unmount(component);
  setLocale('en');
});

it('renders German chrome when the locale is de', async () => {
  aboutInfoMock.mockReset();
  aboutInfoMock.mockResolvedValue(fixture);
  openExternalUrlMock.mockReset();
  const { component, target } = mountPage();

  await waitForLoaded(target);

  setLocale('de');
  await tick();

  const text = target.textContent ?? '';
  expect(text).toContain(resolve('de', 'about.whatsInside'));
  expect(text).toContain(resolve('de', 'about.feature.hybridSearch.title'));
  expect(text).toContain(resolve('de', 'about.feature.hybridSearch.body'));
  expect(text).toContain(resolve('de', 'about.footer'));
  expect(text).not.toContain(resolve('en', 'about.feature.hybridSearch.title'));
  expect(text).not.toContain(resolve('en', 'about.whatsInside'));
  expect(text).not.toContain(resolve('en', 'about.repoCardCta'));

  expect(target.querySelector('.about-link-card-url')?.textContent?.trim()).toBe(
    'github.com/visorcraft/onQ',
  );

  await unmount(component);
  setLocale('en');
});

it('keeps the repo URL line in both locales', async () => {
  aboutInfoMock.mockReset();
  aboutInfoMock.mockResolvedValue(fixture);
  openExternalUrlMock.mockReset();
  const { component, target } = mountPage();

  await waitForLoaded(target);

  expect(target.querySelector('.about-link-card-url')?.textContent?.trim()).toBe(
    'github.com/visorcraft/onQ',
  );

  setLocale('de');
  await tick();

  expect(target.querySelector('.about-link-card-url')?.textContent?.trim()).toBe(
    'github.com/visorcraft/onQ',
  );

  await unmount(component);
  setLocale('en');
});
