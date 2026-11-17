import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

import {
  betaChannel,
  embeddingQuant,
  loadBetaChannel,
  loadEmbeddingQuant,
  loadMinimizeOnCopy,
  loadVaultPreferences,
  minimizeOnCopy,
  setBetaChannel,
  setEmbeddingQuant,
} from './settings';
import { theme } from './theme';

describe('settings store', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    betaChannel.set(false);
    embeddingQuant.set('binary');
    minimizeOnCopy.set(false);
    theme.set('dark');
  });

  // ---------------------------------------------------------------------
  // M7.2 — beta channel opt-in
  // ---------------------------------------------------------------------

  it('persists a beta opt-in through the set_beta_channel command', async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await setBetaChannel(true);

    expect(invokeMock).toHaveBeenCalledWith('set_beta_channel', { enabled: true });
    expect(get(betaChannel)).toBe(true);
  });

  it('persists a beta opt-out and updates the store', async () => {
    betaChannel.set(true);
    invokeMock.mockResolvedValueOnce(undefined);

    await setBetaChannel(false);

    expect(invokeMock).toHaveBeenCalledWith('set_beta_channel', { enabled: false });
    expect(get(betaChannel)).toBe(false);
  });

  it('propagates backend rejections so the UI can roll back', async () => {
    invokeMock.mockRejectedValueOnce(new Error('vault not unlocked'));

    await expect(setBetaChannel(true)).rejects.toThrow('vault not unlocked');
    // Critical: the store must not move when the backend rejects, otherwise
    // the Settings page would silently show the wrong state on next open.
    expect(get(betaChannel)).toBe(false);
  });

  it('loads the persisted beta flag from the backend', async () => {
    invokeMock.mockResolvedValueOnce(true);

    await loadBetaChannel();

    expect(invokeMock).toHaveBeenCalledWith('get_beta_channel');
    expect(get(betaChannel)).toBe(true);
  });

  it('treats a falsy loaded value as the default', async () => {
    invokeMock.mockResolvedValueOnce(false);

    await loadBetaChannel();

    expect(get(betaChannel)).toBe(false);
  });

  it('keeps the default when the backend is unavailable', async () => {
    invokeMock.mockRejectedValueOnce(new Error('vault not unlocked'));

    await loadBetaChannel();

    expect(get(betaChannel)).toBe(false);
  });

  // ---------------------------------------------------------------------
  // M6.10 — embedding quantization (regression coverage so adding the
  // beta block at the top of the file doesn't disturb the existing
  // contract used by Settings page.svelte).
  // ---------------------------------------------------------------------

  it('persists an embedding-quant switch and updates the store', async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await setEmbeddingQuant('dense');

    expect(invokeMock).toHaveBeenCalledWith('set_embedding_quant', { quant: 'dense' });
    expect(get(embeddingQuant)).toBe('dense');
  });

  it('falls back to the default when the embedding-quant column is missing', async () => {
    invokeMock.mockRejectedValueOnce(new Error('vault not unlocked'));

    await loadEmbeddingQuant();

    expect(get(embeddingQuant)).toBe('binary');
  });

  // ---------------------------------------------------------------------
  // minimize_on_copy — palette hide-to-tray preference
  // ---------------------------------------------------------------------

  it('loads minimize_on_copy true from app_state', async () => {
    invokeMock.mockResolvedValueOnce('true');

    await loadMinimizeOnCopy();

    expect(invokeMock).toHaveBeenCalledWith('get_app_setting', {
      key: 'minimize_on_copy',
    });
    expect(get(minimizeOnCopy)).toBe(true);
  });

  it('treats non-true minimize_on_copy values as false', async () => {
    minimizeOnCopy.set(true);
    invokeMock.mockResolvedValueOnce('');

    await loadMinimizeOnCopy();

    expect(get(minimizeOnCopy)).toBe(false);
  });

  it('keeps minimize_on_copy default when the vault is locked', async () => {
    invokeMock.mockRejectedValueOnce(new Error('vault not unlocked'));

    await loadMinimizeOnCopy();

    expect(get(minimizeOnCopy)).toBe(false);
  });

  // ---------------------------------------------------------------------
  // Post-unlock hydration bundle
  // ---------------------------------------------------------------------

  it('loadVaultPreferences hydrates minimize, beta, quant, and theme', async () => {
    invokeMock.mockImplementation((command: string, args?: { key?: string }) => {
      if (command === 'get_app_setting' && args?.key === 'minimize_on_copy') {
        return Promise.resolve('true');
      }
      if (command === 'get_app_setting' && args?.key === 'embedding_quant') {
        return Promise.resolve('dense');
      }
      if (command === 'get_app_setting' && args?.key === 'theme') {
        return Promise.resolve('light');
      }
      if (command === 'get_beta_channel') return Promise.resolve(true);
      if (command === 'set_app_setting') return Promise.resolve(undefined);
      return Promise.resolve(undefined);
    });

    await loadVaultPreferences();

    expect(get(minimizeOnCopy)).toBe(true);
    expect(get(betaChannel)).toBe(true);
    expect(get(embeddingQuant)).toBe('dense');
    expect(get(theme)).toBe('light');
    expect(invokeMock).not.toHaveBeenCalledWith(
      'set_app_setting',
      expect.anything(),
    );
  });
});