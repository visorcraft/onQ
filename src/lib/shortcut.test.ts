import { expect, it } from 'vitest';
import { paletteShortcut } from './shortcut';

it('uses Command only on macOS', () => {
  expect(paletteShortcut('MacIntel')).toBe('⌘K');
  expect(paletteShortcut('Win32')).toBe('Ctrl+K');
  expect(paletteShortcut('Linux x86_64')).toBe('Ctrl+K');
});
