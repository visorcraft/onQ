export type Locale = 'en' | 'th' | 'es' | 'zh' | 'fr' | 'de';

export const SUPPORTED_LOCALES: readonly Locale[] = [
  'en',
  'th',
  'es',
  'zh',
  'fr',
  'de',
] as const;

export const DEFAULT_LOCALE: Locale = 'en';

export const LOCALE_STORAGE_KEY = 'onq.locale';

export function isLocale(value: string): value is Locale {
  return (SUPPORTED_LOCALES as readonly string[]).includes(value);
}
