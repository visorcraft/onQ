import { afterEach, expect, it, vi } from 'vitest';
import { mount, tick, unmount } from 'svelte';
import { version as appVersion } from '../package.json';

const { checkMock } = vi.hoisted(() => ({ checkMock: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/plugin-updater', () => ({ check: checkMock }));

import App from './App.svelte';
import { navigate } from '$lib/stores/navigation';

afterEach(() => {
  vi.useRealTimers();
});

it('checks for updates on startup and from the manual menu item', async () => {
  navigate('home');
  document.body.replaceChildren();
  const target = document.createElement('div');
  document.body.append(target);
  checkMock.mockReset();
  checkMock.mockResolvedValue(null);

  const component = mount(App, { target });

  await vi.waitFor(() => expect(checkMock).toHaveBeenCalledTimes(1));

  const checkUpdates = document.querySelector('button[aria-label="Check for updates"]');
  if (!(checkUpdates instanceof HTMLButtonElement)) {
    throw new Error('Check for updates menu item missing');
  }
  checkUpdates.click();

  await vi.waitFor(() => {
    expect(checkMock).toHaveBeenCalledTimes(2);
    expect(document.querySelector('[role="status"]')?.textContent).toContain(
      'onQ is up to date',
    );
  });

  await unmount(component);
});

it('clears the up-to-date status after five seconds', async () => {
  const scheduled: Array<{ fn: () => void; ms: number }> = [];
  vi.spyOn(globalThis, 'setTimeout').mockImplementation((handler, ms) => {
    if (typeof handler === 'function') {
      scheduled.push({ fn: () => handler(), ms: Number(ms ?? 0) });
    }
    return 0 as unknown as ReturnType<typeof setTimeout>;
  });
  vi.spyOn(globalThis, 'clearTimeout').mockImplementation(() => undefined);

  navigate('home');
  document.body.replaceChildren();
  const target = document.createElement('div');
  document.body.append(target);
  checkMock.mockReset();
  checkMock.mockResolvedValue(null);

  const component = mount(App, { target });
  await vi.waitFor(() => expect(checkMock).toHaveBeenCalledTimes(1));

  document.querySelector<HTMLButtonElement>('button[aria-label="Check for updates"]')?.click();
  await vi.waitFor(() =>
    expect(
      [...document.querySelectorAll('[role="status"]')]
        .map((el) => el.textContent ?? '')
        .join('\n'),
    ).toContain('onQ is up to date'),
  );

  const dismiss = scheduled.find((entry) => entry.ms === 5_000);
  expect(dismiss).toBeDefined();
  dismiss!.fn();
  await tick();

  expect(
    [...document.querySelectorAll('[role="status"]')]
      .map((el) => el.textContent ?? '')
      .join('\n'),
  ).not.toContain('onQ is up to date');

  await unmount(component);
});

it('shows the package version in the bottom-left corner', async () => {
  navigate('home');
  document.body.replaceChildren();
  const target = document.createElement('div');
  document.body.append(target);
  checkMock.mockReset();
  checkMock.mockResolvedValue(null);

  const component = mount(App, { target });
  await vi.waitFor(() => expect(checkMock).toHaveBeenCalledTimes(1));

  const versionEl = document.querySelector(
    '[aria-label="App version, check for updates"]',
  );
  expect(versionEl?.textContent).toBe(`v${appVersion}`);

  await unmount(component);
});

it('only shows update errors for manual checks', async () => {
  navigate('home');
  document.body.replaceChildren();
  const target = document.createElement('div');
  document.body.append(target);
  checkMock.mockReset();
  checkMock.mockRejectedValue(new Error('release feed unavailable'));

  const component = mount(App, { target });

  await vi.waitFor(() => expect(checkMock).toHaveBeenCalledTimes(1));
  expect(
    [...document.querySelectorAll('[role="status"]')].some((el) =>
      el.textContent?.includes('release feed unavailable'),
    ),
  ).toBe(false);

  document.querySelector<HTMLButtonElement>('button[aria-label="Check for updates"]')?.click();

  await vi.waitFor(() =>
    expect(
      [...document.querySelectorAll('[role="status"]')].some((el) =>
        el.textContent?.includes('release feed unavailable'),
      ),
    ).toBe(true),
  );

  await unmount(component);
});
