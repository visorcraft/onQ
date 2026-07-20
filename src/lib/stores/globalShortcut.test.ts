import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  listeners: new Map<string, (event: { payload: string }) => void>(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mocks.invoke,
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(
    async (
      event: string,
      handler: (event: { payload: string }) => void,
    ) => {
      mocks.listeners.set(event, handler);
      return () => mocks.listeners.delete(event);
    },
  ),
}));

import {
  captureGlobalShortcut,
  loadGlobalShortcut,
  matchesGlobalShortcut,
  setGlobalShortcut,
  shortcutFromKeyboardEvent,
} from './globalShortcut';
import { defaultGlobalShortcut, detectShortcutPlatform } from '$lib/shortcut';

function keyboardEvent(
  code: string,
  modifiers: {
    altKey?: boolean;
    ctrlKey?: boolean;
    metaKey?: boolean;
    shiftKey?: boolean;
  } = {},
): KeyboardEvent {
  return new KeyboardEvent('keydown', { code, ...modifiers });
}

beforeEach(() => {
  localStorage.clear();
  mocks.invoke.mockReset();
});

describe('shortcutFromKeyboardEvent', () => {
  it('records a modified letter', () => {
    expect(
      shortcutFromKeyboardEvent(
        keyboardEvent('KeyP', { ctrlKey: true, shiftKey: true }),
      ),
    ).toBe('Ctrl+Shift+P');
  });

  it('records the platform meta key (Meta on Linux, never Super)', () => {
    // Vitest/jsdom CI is Linux-class; Meta is the Arch terminology.
    const platform = detectShortcutPlatform();
    const expected =
      platform === 'mac' ? '⌘+Space' : platform === 'windows' ? 'Win+Space' : 'Meta+Space';
    expect(
      shortcutFromKeyboardEvent(keyboardEvent('Space', { metaKey: true })),
    ).toBe(expected);
    expect(
      shortcutFromKeyboardEvent(keyboardEvent('Space', { metaKey: true })),
    ).not.toMatch(/Super/);
  });

  it('rejects bare keys and modifier-only presses', () => {
    expect(shortcutFromKeyboardEvent(keyboardEvent('KeyP'))).toBeNull();
    expect(
      shortcutFromKeyboardEvent(
        keyboardEvent('ControlLeft', { ctrlKey: true }),
      ),
    ).toBeNull();
  });
});

it('registers a chord with the backend and matches meta+Q in the UI', async () => {
  const display = defaultGlobalShortcut();
  // Linux keeps Meta+…; native platforms convert meta tokens to Super for tauri.
  const expectedBackend =
    detectShortcutPlatform() === 'linux' ? display : 'Super+Q';
  mocks.invoke.mockResolvedValue({
    backend: detectShortcutPlatform() === 'linux' ? 'linux-input' : 'native',
    shortcut: expectedBackend,
  });

  await setGlobalShortcut(display);

  expect(mocks.invoke).toHaveBeenCalledWith('set_global_shortcut', {
    shortcut: expectedBackend,
    interactive: true,
  });
  expect(
    matchesGlobalShortcut(keyboardEvent('KeyQ', { metaKey: true })),
  ).toBe(true);
});

it('captures Linux shortcuts and normalizes Super→Meta in the UI', async () => {
  mocks.invoke.mockResolvedValue({
    backend: 'linux-input',
    shortcut: 'Meta+Q',
  });

  await captureGlobalShortcut();

  expect(mocks.invoke).toHaveBeenCalledWith('capture_global_shortcut');
  expect(
    matchesGlobalShortcut(keyboardEvent('KeyQ', { metaKey: true })),
  ).toBe(true);
});

describe('loadGlobalShortcut', () => {
  it('registers the platform default when nothing is stored (first install)', async () => {
    mocks.invoke.mockImplementation(async (_cmd: string, args: { shortcut?: string | null }) => ({
      backend: detectShortcutPlatform() === 'linux' ? 'linux-input' : 'native',
      shortcut: args.shortcut ?? '',
    }));

    await loadGlobalShortcut();

    const expected = defaultGlobalShortcut();
    const expectedBackend =
      detectShortcutPlatform() === 'linux' ? expected : 'Super+Q';
    expect(mocks.invoke).toHaveBeenCalledWith('set_global_shortcut', {
      shortcut: expectedBackend,
      interactive: false,
    });
    expect(localStorage.getItem('onQ.globalShortcut')).toBe(expected);
    expect(
      matchesGlobalShortcut(keyboardEvent('KeyQ', { metaKey: true })),
    ).toBe(true);
  });

  it('re-registers a stored chord and migrates Super labels in storage', async () => {
    localStorage.setItem('onQ.globalShortcut', 'Super+Space');
    mocks.invoke.mockImplementation(async (_cmd: string, args: { shortcut?: string | null }) => ({
      backend: detectShortcutPlatform() === 'linux' ? 'linux-input' : 'native',
      shortcut: args.shortcut ?? '',
    }));

    await loadGlobalShortcut();

    // UI / localStorage never keeps the legacy Super token.
    const stored = localStorage.getItem('onQ.globalShortcut');
    expect(stored).not.toMatch(/Super/);
    expect(stored?.endsWith('+Space')).toBe(true);
    expect(mocks.invoke).toHaveBeenCalledWith(
      'set_global_shortcut',
      expect.objectContaining({ interactive: false }),
    );
  });
});
