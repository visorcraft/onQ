import { expect, it, vi } from 'vitest';
import { mount, unmount } from 'svelte';

const { checkMock } = vi.hoisted(() => ({ checkMock: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/plugin-updater', () => ({ check: checkMock }));

import App from './App.svelte';

it('checks for updates on startup and from the manual menu item', async () => {
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

it('only shows update errors for manual checks', async () => {
  document.body.replaceChildren();
  const target = document.createElement('div');
  document.body.append(target);
  checkMock.mockReset();
  checkMock.mockRejectedValue(new Error('release feed unavailable'));

  const component = mount(App, { target });

  await vi.waitFor(() => expect(checkMock).toHaveBeenCalledTimes(1));
  expect(document.querySelector('[role="status"]')).toBeNull();

  document.querySelector<HTMLButtonElement>('button[aria-label="Check for updates"]')?.click();

  await vi.waitFor(() =>
    expect(document.querySelector('[role="status"]')?.textContent).toContain(
      'release feed unavailable',
    ),
  );

  await unmount(component);
});
