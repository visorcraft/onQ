import { check } from '@tauri-apps/plugin-updater';
import type { MessageKey } from '$lib/i18n';
import type { Vars } from '$lib/i18n/resolve';

/**
 * Single-source-of-truth for the "Check for updates" action — both the
 * home page footer button and the Settings → Updates panel call into
 * here so the status descriptors, error handling, and the updater
 * endpoint stay in lock-step.
 *
 * `manual` flips the verbosity: silent on app launch (auto-check), loud
 * on user-triggered checks so the user gets either a confirmation that
 * they're up to date or a surfaced error.
 *
 * Returns the final outcome (or the caller renders it via
 * `updateStatusMessage`). The caller decides whether to display it.
 */
export type UpdateCheckOutcome =
  | { kind: 'available'; version: string }
  | { kind: 'up_to_date' }
  | { kind: 'error'; message: string }
  | { kind: 'silent' };

export async function checkForAppUpdates(manual: boolean): Promise<UpdateCheckOutcome> {
  try {
    const update = await check();
    if (update) {
      const version = update.version;
      await update.close();
      return { kind: 'available', version };
    }
    return manual ? { kind: 'up_to_date' } : { kind: 'silent' };
  } catch (error) {
    // Auto-check failures stay silent — surfacing them at launch would
    // alarm users with no updater artefact available (e.g. dev builds).
    // The Settings button is loud; the home badge is quiet.
    return manual
      ? { kind: 'error', message: String(error) }
      : { kind: 'silent' };
  }
}

/** Translatable status descriptor: a catalog key plus optional vars. */
export type UpdateStatusMessage = { key: MessageKey; vars?: Vars };

/**
 * Map an outcome to a translatable descriptor, or `null` when the
 * outcome should stay silent. Render with `t(descriptor.key, descriptor.vars, $locale)`
 * so the text follows the active locale.
 */
export function updateStatusMessage(outcome: UpdateCheckOutcome): UpdateStatusMessage | null {
  switch (outcome.kind) {
    case 'available':
      return { key: 'app.updateAvailable', vars: { version: outcome.version } };
    case 'up_to_date':
      return { key: 'app.upToDate' };
    case 'error':
      return { key: 'app.updateCheckFailed', vars: { message: outcome.message } };
    case 'silent':
      return null;
  }
}
