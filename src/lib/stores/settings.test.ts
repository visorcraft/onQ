import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

import {
  betaChannel,
  embeddingQuant,
  loadBetaChannel,
  loadEmbeddingQuant,
  setBetaChannel,
  setEmbeddingQuant,
} from './settings';

describe('settings store', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    betaChannel.set(false);
    embeddingQuant.set('binary');
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
    // the SettingsPanel would silently show the wrong state on next open.
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
  // contract used by SettingsPanel.svelte).
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
});