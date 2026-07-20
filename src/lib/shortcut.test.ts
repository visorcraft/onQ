import { describe, expect, it } from 'vitest';
import {
  defaultGlobalShortcut,
  detectShortcutPlatform,
  metaModifierLabel,
  normalizeShortcut,
  paletteShortcut,
  toNativeShortcut,
} from './shortcut';

describe('detectShortcutPlatform', () => {
  it('classifies common platform strings', () => {
    expect(detectShortcutPlatform('MacIntel')).toBe('mac');
    expect(detectShortcutPlatform('Win32')).toBe('windows');
    expect(detectShortcutPlatform('Linux x86_64')).toBe('linux');
  });
});

describe('default / palette shortcuts', () => {
  it('uses Win+Q on Windows, Meta+Q on Linux, ⌘+Q on macOS', () => {
    expect(defaultGlobalShortcut('windows')).toBe('Win+Q');
    expect(defaultGlobalShortcut('linux')).toBe('Meta+Q');
    expect(defaultGlobalShortcut('mac')).toBe('⌘+Q');
    expect(paletteShortcut('Win32')).toBe('Win+Q');
    expect(paletteShortcut('Linux x86_64')).toBe('Meta+Q');
    expect(paletteShortcut('MacIntel')).toBe('⌘+Q');
  });

  it('labels the meta key per platform', () => {
    expect(metaModifierLabel('windows')).toBe('Win');
    expect(metaModifierLabel('linux')).toBe('Meta');
    expect(metaModifierLabel('mac')).toBe('⌘');
  });
});

describe('normalizeShortcut / toNativeShortcut', () => {
  it('migrates Super to Meta on Linux and Win on Windows', () => {
    expect(normalizeShortcut('Super+Q', 'linux')).toBe('Meta+Q');
    expect(normalizeShortcut('Super+Space', 'windows')).toBe('Win+Space');
    expect(normalizeShortcut('Super+Q', 'mac')).toBe('⌘+Q');
  });

  it('maps UI meta tokens to Super for tauri/global-hotkey', () => {
    expect(toNativeShortcut('Win+Q')).toBe('Super+Q');
    expect(toNativeShortcut('Meta+Q')).toBe('Super+Q');
    expect(toNativeShortcut('⌘+Q')).toBe('Super+Q');
    expect(toNativeShortcut('Ctrl+Shift+P')).toBe('Ctrl+Shift+P');
  });
});
