import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

import {
  checkAndStart,
  complete,
  next,
  prev,
  reset,
  startIfNeeded,
  tutorialStep,
  tutorialVisible,
} from './tutorial';

describe('tutorial store', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    tutorialStep.set(0);
    tutorialVisible.set(false);
  });

  it('shows an incomplete tutorial', async () => {
    invokeMock.mockResolvedValueOnce('false');

    await startIfNeeded();

    expect(invokeMock).toHaveBeenCalledWith('get_app_setting', {
      key: 'tutorial_completed',
    });
    expect(get(tutorialVisible)).toBe(true);
  });

  it('keeps a completed tutorial hidden', async () => {
    invokeMock.mockResolvedValueOnce('true');

    await startIfNeeded();

    expect(get(tutorialVisible)).toBe(false);
  });

  it('shows the tutorial when the completion setting is missing', async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await startIfNeeded();

    expect(get(tutorialVisible)).toBe(true);
  });

  it('checks for an incomplete tutorial only when the vault is ready', async () => {
    invokeMock.mockResolvedValueOnce('false');

    await checkAndStart(false);
    expect(invokeMock).not.toHaveBeenCalled();

    await checkAndStart(true);

    expect(invokeMock).toHaveBeenCalledWith('get_app_setting', {
      key: 'tutorial_completed',
    });
    expect(get(tutorialVisible)).toBe(true);
  });

  it('persists completion after hiding the tutorial', async () => {
    tutorialVisible.set(true);
    invokeMock.mockResolvedValueOnce(undefined);

    await complete();

    expect(invokeMock).toHaveBeenCalledWith('set_app_setting', {
      key: 'tutorial_completed',
      value: 'true',
    });
    expect(get(tutorialVisible)).toBe(false);
  });

  it('hides the tutorial when completion cannot be persisted', async () => {
    tutorialVisible.set(true);
    invokeMock.mockRejectedValueOnce(new Error('write failed'));

    await expect(complete()).resolves.toBeUndefined();

    expect(get(tutorialVisible)).toBe(false);
  });

  it('keeps next and previous steps within the four-step range', () => {
    prev();
    expect(get(tutorialStep)).toBe(0);

    next();
    next();
    next();
    next();
    expect(get(tutorialStep)).toBe(3);

    prev();
    expect(get(tutorialStep)).toBe(2);
  });

  it('resets to the first step and opens the tutorial', () => {
    tutorialStep.set(3);

    reset();

    expect(get(tutorialStep)).toBe(0);
    expect(get(tutorialVisible)).toBe(true);
  });
});
