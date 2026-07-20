import { describe, expect, it, vi } from 'vitest';

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
  matchesGlobalShortcut,
  setGlobalShortcut,
  shortcutFromKeyboardEvent,
} from './globalShortcut';

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

describe('shortcutFromKeyboardEvent', () => {
  it('records a modified letter', () => {
    expect(
      shortcutFromKeyboardEvent(
        keyboardEvent('KeyP', { ctrlKey: true, shiftKey: true }),
      ),
    ).toBe('Ctrl+Shift+P');
  });

  it('records Super without a platform-specific glyph', () => {
    expect(
      shortcutFromKeyboardEvent(keyboardEvent('Space', { metaKey: true })),
    ).toBe('Super+Space');
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

it('registers once in Rust and keeps that grab out of the local key handler', async () => {
  mocks.invoke.mockResolvedValue({
    backend: 'native',
    shortcut: 'Ctrl+Q',
  });

  await setGlobalShortcut('Ctrl+Q');

  expect(mocks.invoke).toHaveBeenCalledWith('set_global_shortcut', {
    shortcut: 'Ctrl+Q',
    interactive: true,
  });
  expect(
    matchesGlobalShortcut(keyboardEvent('KeyQ', { ctrlKey: true })),
  ).toBe(true);
});

it('captures Linux shortcuts in the backend listener', async () => {
  mocks.invoke.mockResolvedValue({
    backend: 'linux-input',
    shortcut: 'Super+Q',
  });

  await captureGlobalShortcut();

  expect(mocks.invoke).toHaveBeenCalledWith('capture_global_shortcut');
  expect(
    matchesGlobalShortcut(keyboardEvent('KeyQ', { metaKey: true })),
  ).toBe(true);
});
