import { check } from '@tauri-apps/plugin-updater';

/**
 * Single-source-of-truth for the "Check for updates" action — both the
 * home page footer button and the Settings → Updates panel call into
 * here so the status strings, error handling, and the updater endpoint
 * stay in lock-step.
 *
 * `manual` flips the verbosity: silent on app launch (auto-check), loud
 * on user-triggered checks so the user gets either a confirmation that
 * they're up to date or a surfaced error.
 *
 * Returns the final status string (or `null` for a silent auto-check
 * with no update available). The caller decides whether to display it.
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

/** Convenience helper for surfaces that just want a human-readable string. */
export function formatUpdateStatus(outcome: UpdateCheckOutcome): string | null {
  switch (outcome.kind) {
    case 'available':
      return `onQ ${outcome.version} is available`;
    case 'up_to_date':
      return 'onQ is up to date';
    case 'error':
      return `Unable to check for updates: ${outcome.message}`;
    case 'silent':
      return null;
  }
}