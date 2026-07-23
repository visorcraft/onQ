/** English message catalog (i18n scaffold). */
export const en = {
  'palette.placeholder': 'Search prompts, or type to create…',
  'palette.lockVault': 'Lock vault',
  'settings.autoLock': 'Auto-lock',
  'settings.recency': 'Search recency half-life (days)',
  'settings.historyRetention': 'History retention (days)',
  'editor.history': 'History',
  'editor.suggestTags': 'Suggested tags',
  'backup.remind': 'Vault backup is overdue',
} as const;

export type MessageKey = keyof typeof en;

export function t(key: MessageKey): string {
  return en[key] ?? key;
}
