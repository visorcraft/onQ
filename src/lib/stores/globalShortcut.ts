import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { writable } from 'svelte/store';
import {
  defaultGlobalShortcut,
  detectShortcutPlatform,
  metaModifierLabel,
  normalizeShortcut,
  toNativeShortcut,
} from '$lib/shortcut';

const STORAGE_KEY = 'onQ.globalShortcut';
const PRESSED_EVENT = 'onq-global-shortcut';

type ShortcutBackend = 'native' | 'linux-input';
type ShortcutStatus = {
  backend: ShortcutBackend;
  shortcut: string;
};

export const globalShortcut = writable('');
export const globalShortcutBackend = writable<ShortcutBackend>('native');

let currentShortcut = '';
let listeners: Promise<void> | undefined;

function applyDisplayShortcut(display: string, backend: ShortcutBackend) {
  currentShortcut = display;
  globalShortcut.set(display);
  globalShortcutBackend.set(backend);
  if (display) localStorage.setItem(STORAGE_KEY, display);
  else localStorage.removeItem(STORAGE_KEY);
}

function ensureListeners(): Promise<void> {
  listeners ??= listen('global-shortcut-pressed', () => {
    window.dispatchEvent(new Event(PRESSED_EVENT));
  }).then(() => undefined);
  return listeners;
}

async function updateShortcut(
  shortcut: string | null,
  interactive: boolean,
): Promise<void> {
  await ensureListeners();
  // Always register a concrete chord with the backend. Empty/null must not
  // leave the tray/global grab unset (first-install bug).
  const display = shortcut && shortcut.length > 0 ? normalizeShortcut(shortcut) : null;
  // Linux matches exact UI strings (Meta+…); Windows/macOS tauri only parses Super.
  const forBackend = display
    ? detectShortcutPlatform() === 'linux'
      ? display
      : toNativeShortcut(display)
    : null;
  const status = await invoke<ShortcutStatus>('set_global_shortcut', {
    shortcut: forBackend,
    interactive,
  });
  // Prefer our platform-facing label over whatever the native backend echoes
  // (it typically normalizes the meta key to Super).
  const resolved =
    display ??
    (status.shortcut ? normalizeShortcut(status.shortcut) : '');
  applyDisplayShortcut(resolved, status.backend);
}

export async function setGlobalShortcut(shortcut: string): Promise<void> {
  await updateShortcut(shortcut, true);
}

export async function captureGlobalShortcut(): Promise<void> {
  await ensureListeners();
  const status = await invoke<ShortcutStatus>('capture_global_shortcut');
  const display = status.shortcut ? normalizeShortcut(status.shortcut) : '';
  applyDisplayShortcut(display, status.backend);
}

/**
 * Load the saved chord, or the platform default on first install, and
 * register it with the native / Linux input backend.
 */
export async function loadGlobalShortcut(): Promise<void> {
  const stored = localStorage.getItem(STORAGE_KEY);
  const shortcut =
    stored && stored.trim().length > 0
      ? normalizeShortcut(stored)
      : defaultGlobalShortcut();
  await updateShortcut(shortcut, false);
}

export function matchesGlobalShortcut(event: KeyboardEvent): boolean {
  return shortcutFromKeyboardEvent(event) === currentShortcut;
}

export const globalShortcutPressedEvent = PRESSED_EVENT;

const specialKeys: Record<string, string> = {
  ArrowDown: 'ArrowDown',
  ArrowLeft: 'ArrowLeft',
  ArrowRight: 'ArrowRight',
  ArrowUp: 'ArrowUp',
  Backquote: 'Backquote',
  Backslash: 'Backslash',
  Backspace: 'Backspace',
  BracketLeft: 'BracketLeft',
  BracketRight: 'BracketRight',
  Comma: 'Comma',
  Delete: 'Delete',
  End: 'End',
  Enter: 'Enter',
  Equal: 'Equal',
  Home: 'Home',
  Insert: 'Insert',
  Minus: 'Minus',
  PageDown: 'PageDown',
  PageUp: 'PageUp',
  Period: 'Period',
  Quote: 'Quote',
  Semicolon: 'Semicolon',
  Slash: 'Slash',
  Space: 'Space',
  Tab: 'Tab',
};

export function shortcutFromKeyboardEvent(event: KeyboardEvent): string | null {
  if (!event.ctrlKey && !event.altKey && !event.metaKey) return null;

  let key: string | undefined;
  if (/^Key[A-Z]$/.test(event.code)) key = event.code.slice(3);
  else if (/^Digit[0-9]$/.test(event.code)) key = event.code.slice(5);
  else if (/^F(?:[1-9]|1[0-9]|2[0-4])$/.test(event.code)) key = event.code;
  else key = specialKeys[event.code];
  if (!key) return null;

  return [
    event.ctrlKey && 'Ctrl',
    event.altKey && 'Alt',
    event.shiftKey && 'Shift',
    event.metaKey && metaModifierLabel(),
    key,
  ]
    .filter(Boolean)
    .join('+');
}
