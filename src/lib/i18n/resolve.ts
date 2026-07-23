import { catalogs } from './catalogs';
import type { MessageKey } from './en';
import { en } from './en';
import { DEFAULT_LOCALE, isLocale, type Locale } from './types';

export type Vars = Record<string, string | number>;

/**
 * Resolve a message for `locale`, falling back to English then the key.
 * Interpolates `{name}` placeholders from `vars`.
 */
export function resolve(
  locale: Locale,
  key: MessageKey | string,
  vars?: Vars,
): string {
  const catalog = catalogs[locale] ?? catalogs[DEFAULT_LOCALE];
  let text: string;
  if (key in en) {
    const k = key as MessageKey;
    text = catalog[k] ?? en[k] ?? key;
  } else {
    text = key;
  }
  if (vars) {
    for (const [name, value] of Object.entries(vars)) {
      text = text.replaceAll(`{${name}}`, String(value));
    }
  }
  return text;
}

/** Map navigator/OS language tags to a supported Locale. */
export function matchNavigatorLanguage(
  languages: readonly string[],
): Locale | null {
  for (const raw of languages) {
    const tag = raw.toLowerCase().replace('_', '-');
    const primary = tag.split('-')[0] ?? tag;
    if (primary === 'zh') return 'zh'; // Simplified for this product
    if (isLocale(primary)) return primary;
  }
  return null;
}

export function missingKeys(locale: Locale): MessageKey[] {
  if (locale === 'en') return [];
  const catalog = catalogs[locale];
  return (Object.keys(en) as MessageKey[]).filter(
    (k) => !catalog[k] || catalog[k].trim() === '',
  );
}
