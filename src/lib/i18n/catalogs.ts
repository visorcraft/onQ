import { en, type MessageKey } from './en';
import { th } from './th';
import { es } from './es';
import { zh } from './zh';
import { fr } from './fr';
import { de } from './de';
import type { Locale } from './types';

export type Catalog = Record<MessageKey, string>;

export const catalogs: Record<Locale, Catalog> = {
  en: en as Catalog,
  th,
  es,
  zh,
  fr,
  de,
};

/** All product keys from the English source of truth. */
export const ALL_KEYS: MessageKey[] = Object.keys(en) as MessageKey[];
