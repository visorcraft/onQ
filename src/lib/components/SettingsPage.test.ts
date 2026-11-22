import { afterEach, describe, expect, it, vi } from 'vitest';
import { mount, tick, unmount } from 'svelte';

const { invokeMock, checkMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
  checkMock: vi.fn(),
}));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));
vi.mock('@tauri-apps/plugin-updater', () => ({ check: checkMock }));

import SettingsPage from './SettingsPage.svelte';
import { resolve, setLocale } from '$lib/i18n';

afterEach(() => {
  document.body.replaceChildren();
  setLocale('en');
  vi.restoreAllMocks();
});

describe('SettingsPage updates status region', () => {
  it('shows the localized up-to-date status', async () => {
    setLocale('de');
    checkMock.mockResolvedValue(null);
    const target = document.createElement('div');
    document.body.append(target);

    const component = mount(SettingsPage, { target });
    await tick();

    const updatesNav = [...target.querySelectorAll<HTMLButtonElement>('.settings-nav .nav-item')].find(
      (button) => button.textContent?.includes(resolve('de', 'settings.updates')),
    );
    if (!updatesNav) throw new Error('updates nav item missing');
    updatesNav.click();
    await tick();

    const checkButton = target.querySelector<HTMLButtonElement>('button.control-btn.primary');
    if (!checkButton) throw new Error('check-for-updates button missing');
    checkButton.click();

    await vi.waitFor(() => {
      const status = target.querySelector('[role="status"]');
      expect(status?.textContent).toContain(resolve('de', 'app.upToDate'));
    });

    await unmount(component);
  });
});
