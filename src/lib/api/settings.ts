import { invoke } from '@tauri-apps/api/core';

/**
 * Read a per-vault UI preference from the encrypted search-index DB's
 * singleton `app_state` row. Returns the empty string when the column is
 * unset so the caller can apply its own default without having to know
 * which keys were ever populated.
 *
 * The backend's `setting_column_for_key` enforces a closed set of keys,
 * so unknown keys reject with a typed error — propagate it verbatim
 * instead of pretending the value is empty.
 */
export async function getAppSetting(key: string): Promise<string> {
  return invoke<string>('get_app_setting', { key });
}

/**
 * Write a per-vault UI preference. Preserves every other column on the
 * row, so partial writes don't blow away neighbouring preferences.
 */
export async function setAppSetting(key: string, value: string): Promise<void> {
  return invoke('set_app_setting', { key, value });
}
