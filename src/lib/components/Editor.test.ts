import { afterEach, describe, expect, it, vi } from 'vitest';
import { mount, unmount } from 'svelte';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

import Editor from './Editor.svelte';
import { resolve, setLocale } from '$lib/i18n';

afterEach(() => {
  document.body.replaceChildren();
  setLocale('en');
  vi.restoreAllMocks();
});

describe('Editor loading state', () => {
  it('shows the localized loading message', async () => {
    invokeMock.mockImplementation(() => new Promise(() => {}));
    setLocale('de');
    const target = document.createElement('div');
    document.body.append(target);

    const component = mount(Editor, {
      target,
      props: { id: 'p1', onClose: () => {} },
    });

    await vi.waitFor(() => {
      expect(target.querySelector('.loading-state p')).toBeTruthy();
    });

    const loading = target.querySelector('.loading-state p');
    expect(loading?.textContent?.trim()).toBe(resolve('de', 'editor.loadingPrompt'));
    expect(loading?.textContent).not.toContain('Loading prompt');

    await unmount(component);
  });
});
