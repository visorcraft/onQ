import { beforeEach, describe, expect, it } from 'vitest';
import { ALL_KEYS, catalogs } from './catalogs';
import { loadLocale, readStoredLocale, setLocale, getLocale, t } from './index';
import { matchNavigatorLanguage, missingKeys, resolve } from './resolve';
import { LOCALE_STORAGE_KEY, SUPPORTED_LOCALES, type Locale } from './types';
import type { MessageKey } from '$lib/i18n';

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

describe('palette.templateValuePrompt', () => {
  it('exists in all six catalogs', () => {
    for (const loc of SUPPORTED_LOCALES) {
      expect(
        catalogs[loc]['palette.templateValuePrompt'].trim().length,
        `missing palette.templateValuePrompt in ${loc}`,
      ).toBeGreaterThan(0);
    }
  });

  it('formats the field name inside literal braces', () => {
    expect(
      resolve('en', 'palette.templateValuePrompt', { name: 'client_name' }),
    ).toBe('Value for {{client_name}}');
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

  it.each(SUPPORTED_LOCALES.filter((l) => l !== 'en') as Locale[])(
    '%s uses the same {placeholders} as English for every key',
    (loc) => {
      const names = (s: string) =>
        (s.match(/\{[A-Za-z_]+\}/g) ?? []).sort().join(',');
      for (const key of ALL_KEYS) {
        expect(
          names(catalogs[loc][key]),
          `placeholder mismatch for ${key} in ${loc}`,
        ).toBe(names(catalogs.en[key]));
      }
    },
  );

  it.each(SUPPORTED_LOCALES)('resolves the new chrome keys in %s', (loc) => {
    const newKeys = [
      'pager.label',
      'pager.previous',
      'pager.next',
      'common.closeDialog',
      'common.dialog',
      'tutorial.dialogAria',
      'tutorial.finish',
      'settings.shortcutRecord',
      'settings.shortcutRecording',
      'editor.loadingPrompt',
      'editor.title',
      'smart.fieldTag',
      'smart.fieldFolder',
      'smart.fieldFavorite',
      'smart.fieldLocked',
      'smart.fieldText',
      'smart.opIs',
      'smart.opNot',
      'smart.opContains',
      'smart.valueTrue',
      'smart.valueFalse',
      'library.renameLeafOnly',
      'library.newPromptEditorUnavailable',
    ] as const;
    for (const key of newKeys) {
      const got = resolve(loc, key);
      expect(got.trim().length).toBeGreaterThan(0);
      expect(got).not.toBe(key);
    }
    const stepOf = resolve(loc, 'tutorial.stepOf', { current: 2, total: 4 });
    expect(stepOf).not.toContain('{current}');
    expect(stepOf).not.toContain('{total}');
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

const canon = (s: string) => s.trim().replace(/\s+/g, ' ').toLowerCase();

const SHARED_WITH_ENGLISH = {
  th: ['about.credits.col.crate', 'locale.de', 'locale.en', 'locale.es', 'locale.fr', 'locale.th', 'locale.zh', 'settings.palette'],
  es: ['about.credits.col.crate', 'about.eyebrow', 'about.feature.plugins.title', 'diff.base', 'editor.editorTab', 'editor.promptHeading', 'library.kindPrompt', 'locale.de', 'locale.en', 'locale.es', 'locale.fr', 'locale.th', 'locale.zh', 'palette.prompts', 'plugins.filterName', 'settings.general', 'settings.palette', 'settings.plugins', 'tutorial.dialogAria'],
  zh: ['about.credits.col.crate', 'locale.de', 'locale.en', 'locale.es', 'locale.fr', 'locale.th', 'locale.zh', 'settings.palette'],
  fr: ['about.credits.col.crate', 'about.credits.col.version', 'about.eyebrow', 'about.feature.plugins.title', 'common.ok', 'diff.base', 'editor.promptHeading', 'library.kindPrompt', 'locale.de', 'locale.en', 'locale.es', 'locale.fr', 'locale.th', 'locale.zh', 'pager.label', 'palette.prompts', 'plugins.filterName', 'settings.dense', 'settings.palette', 'settings.plugins'],
  de: ['about.credits.col.crate', 'about.credits.col.version', 'about.eyebrow', 'about.feature.plugins.title', 'common.dialog', 'common.ok', 'editor.editorTab', 'editor.promptHeading', 'editor.tags', 'library.kindPrompt', 'library.name', 'library.tags', 'locale.de', 'locale.en', 'locale.es', 'locale.fr', 'locale.th', 'locale.zh', 'palette.prompts', 'plugins.filterName', 'settings.backups', 'settings.dense', 'settings.palette', 'settings.plugins', 'settings.updates', 'smart.fieldTag', 'smart.fieldText', 'smart.operator', 'tutorial.dialogAria'],
} satisfies Record<Locale, readonly MessageKey[]>;

describe('untranslated catalog values', () => {
  it.each(
    SUPPORTED_LOCALES.filter((l) => l !== 'en') as Exclude<Locale, 'en'>[],
  )('%s shares English values only for intentional cognates', (loc) => {
    const allow: readonly MessageKey[] = SHARED_WITH_ENGLISH[loc];
    const shared = ALL_KEYS.filter(
      (key) => canon(catalogs[loc][key]) === canon(catalogs.en[key]),
    );
    const unexpected = shared.filter((key) => !allow.includes(key));
    expect(
      unexpected,
      `untranslated (identical to English) in ${loc}: ${unexpected.join(', ')}`,
    ).toEqual([]);
    const stale = allow.filter((key) => !shared.includes(key));
    expect(
      stale,
      `stale allow-list entry in ${loc}: ${stale.join(', ')}`,
    ).toEqual([]);
  });
});
