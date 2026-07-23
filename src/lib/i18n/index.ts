/**
 * onQ i18n entry: `t(key)`, locale store, catalogs for en/th/es/zh/fr/de.
 */
import { get } from 'svelte/store';
import type { MessageKey } from './en';
import { locale } from './locale';
import { resolve, type Vars } from './resolve';
import type { Locale } from './types';

export type { MessageKey } from './en';
export type { Locale } from './types';
export { SUPPORTED_LOCALES, DEFAULT_LOCALE, isLocale } from './types';
export {
  locale,
  getLocale,
  setLocale,
  loadLocale,
  readStoredLocale,
} from './locale';
export { resolve, matchNavigatorLanguage, missingKeys } from './resolve';
export { catalogs, ALL_KEYS } from './catalogs';

/**
 * Translate `key` using the active locale store.
 * For reactive UI, call inside a template that also reads `$locale`
 * (or pass `loc` explicitly after `$locale` is in scope).
 */
export function t(key: MessageKey, vars?: Vars, loc?: Locale): string {
  const active = loc ?? get(locale);
  return resolve(active, key, vars);
}
