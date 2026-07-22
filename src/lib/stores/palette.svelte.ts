// Shared open/close control for the global command palette so non-palette
// surfaces (home card, app chrome) can open it without coupling to the
// component instance.
let _open = $state(false);

export const palette = {
  get open() {
    return _open;
  },
};

export function openPalette() {
  _open = true;
}

export function closePalette() {
  _open = false;
}

export function togglePalette() {
  _open = !_open;
}