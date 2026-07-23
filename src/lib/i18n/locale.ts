import { get, writable } from 'svelte/store';
import {
  DEFAULT_LOCALE,
  isLocale,
  LOCALE_STORAGE_KEY,
  type Locale,
} from './types';
import { matchNavigatorLanguage } from './resolve';

/** Active UI locale. Subscribe so Svelte components re-render on change. */
export const locale = writable<Locale>(DEFAULT_LOCALE);

export function getLocale(): Locale {
  return get(locale);
}

/**
 * Persist locale to localStorage (works before vault unlock) and update store.
 */
export function setLocale(next: Locale): void {
  locale.set(next);
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(LOCALE_STORAGE_KEY, next);
    }
  } catch {
    // private mode / blocked storage — store still updates in-memory
  }
}

/** Read stored preference without mutating. */
export function readStoredLocale(): Locale | null {
  try {
    if (typeof localStorage === 'undefined') return null;
    const raw = localStorage.getItem(LOCALE_STORAGE_KEY);
    if (raw && isLocale(raw)) return raw;
  } catch {
    /* ignore */
  }
  return null;
}

/**
 * Hydrate locale: stored preference → navigator match → English default.
 * Call once at app boot.
 */
export function loadLocale(): Locale {
  const stored = readStoredLocale();
  if (stored) {
    locale.set(stored);
    return stored;
  }
  const langs =
    typeof navigator !== 'undefined' && navigator.languages?.length
      ? navigator.languages
      : typeof navigator !== 'undefined' && navigator.language
        ? [navigator.language]
        : [];
  const matched = matchNavigatorLanguage(langs);
  const next = matched ?? DEFAULT_LOCALE;
  locale.set(next);
  return next;
}
