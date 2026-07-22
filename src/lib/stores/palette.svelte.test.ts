import { describe, expect, it, beforeEach } from 'vitest';
import {
  palette,
  openPalette,
  closePalette,
  togglePalette,
} from './palette.svelte';

describe('palette store', () => {
  beforeEach(() => {
    closePalette();
  });

  it('starts closed', () => {
    expect(palette.open).toBe(false);
  });

  it('opens via openPalette()', () => {
    openPalette();
    expect(palette.open).toBe(true);
  });

  it('closes via closePalette()', () => {
    openPalette();
    closePalette();
    expect(palette.open).toBe(false);
  });

  it('toggles open -> closed -> open', () => {
    togglePalette();
    expect(palette.open).toBe(true);
    togglePalette();
    expect(palette.open).toBe(false);
    togglePalette();
    expect(palette.open).toBe(true);
  });
});