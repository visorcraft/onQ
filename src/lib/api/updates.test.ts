import { beforeEach, describe, expect, it, vi } from 'vitest';

const { checkMock } = vi.hoisted(() => ({ checkMock: vi.fn() }));
vi.mock('@tauri-apps/plugin-updater', () => ({ check: checkMock }));

import {
  checkForAppUpdates,
  updateStatusMessage,
} from './updates';
import { resolve } from '$lib/i18n/resolve';
import { SUPPORTED_LOCALES } from '$lib/i18n/types';

beforeEach(() => {
  checkMock.mockReset();
});

describe('checkForAppUpdates', () => {
  it('returns available and closes the update exactly once', async () => {
    const close = vi.fn().mockResolvedValue(undefined);
    checkMock.mockResolvedValue({ version: '1.15.0', close });

    const outcome = await checkForAppUpdates(true);

    expect(outcome).toEqual({ kind: 'available', version: '1.15.0' });
    expect(close).toHaveBeenCalledTimes(1);
  });

  it('returns up_to_date for manual and silent for auto when no update', async () => {
    checkMock.mockResolvedValue(null);

    await expect(checkForAppUpdates(true)).resolves.toEqual({ kind: 'up_to_date' });
    await expect(checkForAppUpdates(false)).resolves.toEqual({ kind: 'silent' });
  });

  it('returns error for manual and silent for auto when check rejects', async () => {
    checkMock.mockRejectedValue(new Error('offline'));

    await expect(checkForAppUpdates(true)).resolves.toEqual({
      kind: 'error',
      message: 'Error: offline',
    });
    await expect(checkForAppUpdates(false)).resolves.toEqual({ kind: 'silent' });
  });
});

describe('updateStatusMessage', () => {
  it('maps each outcome kind to the expected descriptor', () => {
    expect(
      updateStatusMessage({ kind: 'available', version: '1.15.0' }),
    ).toEqual({ key: 'app.updateAvailable', vars: { version: '1.15.0' } });

    expect(updateStatusMessage({ kind: 'up_to_date' })).toEqual({
      key: 'app.upToDate',
    });

    expect(
      updateStatusMessage({ kind: 'error', message: 'boom' }),
    ).toEqual({ key: 'app.updateCheckFailed', vars: { message: 'boom' } });

    expect(updateStatusMessage({ kind: 'silent' })).toBeNull();
  });
});

describe('update status keys', () => {
  it.each(SUPPORTED_LOCALES)('%s resolves the three keys', (locale) => {
    const available = resolve(locale, 'app.updateAvailable', { version: '9.9.9' });
    expect(available).toContain('9.9.9');
    expect(available).not.toBe('app.updateAvailable');

    const upToDate = resolve(locale, 'app.upToDate');
    expect(upToDate.trim().length).toBeGreaterThan(0);
    expect(upToDate).not.toBe('app.upToDate');

    const failed = resolve(locale, 'app.updateCheckFailed', { message: 'bad {version}' });
    expect(failed.trim().length).toBeGreaterThan(0);
    expect(failed).not.toBe('app.updateCheckFailed');
    // Braces inside the error message are emitted verbatim (single pass).
    expect(failed).toContain('bad {version}');
  });
});
