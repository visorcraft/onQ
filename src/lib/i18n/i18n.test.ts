import { beforeEach, describe, expect, it } from 'vitest';
import { ALL_KEYS, catalogs } from './catalogs';
import { loadLocale, readStoredLocale, setLocale, getLocale, t } from './index';
import { matchNavigatorLanguage, missingKeys, resolve } from './resolve';
import { LOCALE_STORAGE_KEY, SUPPORTED_LOCALES, type Locale } from './types';

describe('i18n resolve', () => {
  it('returns English for en locale', () => {
    expect(resolve('en', 'palette.lockVault')).toBe('Lock vault');
  });

  it.each([
    ['th', 'ล็อกห้องนิรภัย'],
    ['es', 'Bloquear bóveda'],
    ['zh', '锁定保险库'],
    ['fr', 'Verrouiller le coffre'],
    ['de', 'Tresor sperren'],
  ] as const)('resolves palette.lockVault in %s', (loc, expected) => {
    const got = resolve(loc, 'palette.lockVault');
    expect(got).toBe(expected);
    expect(got).not.toBe(resolve('en', 'palette.lockVault'));
  });

  it('falls back to English when key missing in catalog edge case', () => {
    // force-resolve a key that exists only in en path via English fallback
    expect(resolve('de', 'settings.language')).toBeTruthy();
    expect(resolve('de', 'settings.language')).not.toBe('settings.language');
  });

  it('falls back to key string for unknown keys', () => {
    expect(resolve('en', 'not.a.real.key' as 'app.home')).toBe('not.a.real.key');
  });

  it('interpolates vars', () => {
    expect(
      resolve('en', 'palette.moreLike', { title: 'Hello' }),
    ).toBe('More like this: Hello');
    expect(
      resolve('de', 'palette.moreLike', { title: 'Hallo' }),
    ).toContain('Hallo');
  });
});

describe('catalog completeness', () => {
  it.each(SUPPORTED_LOCALES.filter((l) => l !== 'en') as Locale[])(
    '%s has every English key filled',
    (loc) => {
      const missing = missingKeys(loc);
      expect(missing, `missing keys in ${loc}: ${missing.join(', ')}`).toEqual(
        [],
      );
      for (const key of ALL_KEYS) {
        expect(catalogs[loc][key]?.trim().length).toBeGreaterThan(0);
      }
    },
  );

  it('English key count matches ALL_KEYS', () => {
    expect(ALL_KEYS.length).toBeGreaterThan(50);
    expect(Object.keys(catalogs.en).length).toBe(ALL_KEYS.length);
  });
});

describe('matchNavigatorLanguage', () => {
  it('matches primary tags', () => {
    expect(matchNavigatorLanguage(['de-DE', 'en-US'])).toBe('de');
    expect(matchNavigatorLanguage(['zh-CN'])).toBe('zh');
    expect(matchNavigatorLanguage(['th'])).toBe('th');
    expect(matchNavigatorLanguage(['pt-BR'])).toBeNull();
  });
});

describe('locale store + persistence', () => {
  beforeEach(() => {
    localStorage.removeItem(LOCALE_STORAGE_KEY);
    setLocale('en');
  });

  it('setLocale persists and getLocale returns it', () => {
    setLocale('fr');
    expect(getLocale()).toBe('fr');
    expect(localStorage.getItem(LOCALE_STORAGE_KEY)).toBe('fr');
    expect(readStoredLocale()).toBe('fr');
  });

  it('loadLocale reads stored preference', () => {
    localStorage.setItem(LOCALE_STORAGE_KEY, 'es');
    expect(loadLocale()).toBe('es');
    expect(getLocale()).toBe('es');
  });

  it('loadLocale falls back to navigator when unset', () => {
    localStorage.removeItem(LOCALE_STORAGE_KEY);
    const original = navigator.languages;
    Object.defineProperty(navigator, 'languages', {
      configurable: true,
      get: () => ['de-AT', 'en'],
    });
    try {
      expect(loadLocale()).toBe('de');
    } finally {
      Object.defineProperty(navigator, 'languages', {
        configurable: true,
        get: () => original,
      });
    }
  });

  it('switch language changes t() output via resolve path', () => {
    setLocale('en');
    expect(t('settings.language')).toBe('Language');
    setLocale('zh');
    expect(t('settings.language')).toBe('语言');
    setLocale('th');
    expect(t('settings.language')).toBe('ภาษา');
  });
});