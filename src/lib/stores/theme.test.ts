import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

import { loadTheme, setTheme, theme } from './theme';

describe('theme store', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    theme.set('dark');
    document.documentElement.classList.remove('light', 'dark');
  });

  it('loadTheme paints a stored theme without writing the search index', async () => {
    invokeMock.mockResolvedValueOnce('light');

    await loadTheme();

    expect(get(theme)).toBe('light');
    expect(document.documentElement.classList.contains('light')).toBe(true);
    expect(invokeMock).toHaveBeenCalledTimes(1);
    expect(invokeMock).toHaveBeenCalledWith('get_app_setting', { key: 'theme' });
  });

  it('loadTheme paints dark for an empty or unknown value and does not write', async () => {
    invokeMock.mockResolvedValueOnce('');

    await loadTheme();

    expect(get(theme)).toBe('dark');
    expect(invokeMock).not.toHaveBeenCalledWith(
      'set_app_setting',
      expect.anything(),
    );

    invokeMock.mockReset();
    theme.set('light');
    invokeMock.mockResolvedValueOnce('blue');

    await loadTheme();

    expect(get(theme)).toBe('dark');
    expect(invokeMock).not.toHaveBeenCalledWith(
      'set_app_setting',
      expect.anything(),
    );
  });

  it('setTheme still persists an explicit toggle', async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await setTheme('light');

    expect(get(theme)).toBe('light');
    expect(invokeMock).toHaveBeenCalledWith('set_app_setting', {
      key: 'theme',
      value: 'light',
    });
  });
});
