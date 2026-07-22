// Shared open/close control for the global command palette so non-palette
// surfaces (home card, app chrome) can open it without coupling to the
// component instance.
let _open = $state(false);
let _suppressNextOpen = 0;

export const palette = {
  get open() {
    return _open;
  },
};

export function openPalette() {
  // Coalesce duplicate opens (e.g. shortcut fired while the palette is
  // already animating in) so the input ref doesn't get re-queried mid-tick.
  if (_open && _suppressNextOpen > 0) {
    _suppressNextOpen -= 1;
    return;
  }
  _open = true;
  if (_suppressNextOpen > 0) _suppressNextOpen -= 1;
}

export function closePalette() {
  _open = false;
  _suppressNextOpen = 0;
}

export function togglePalette() {
  if (_open) closePalette();
  else openPalette();
}

/** Called by the palette right after it has just opened, to suppress a
 * duplicate open fired from the same originating event. */
export function notePaletteOpened() {
  _suppressNextOpen = 1;
}