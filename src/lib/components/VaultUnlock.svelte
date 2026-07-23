<script lang="ts">
  import { recoverVault, unlockVault } from '$lib/api/vault';
  import { t, locale } from '$lib/i18n';

  let {
    path,
    mode,
    onVaultReady,
    onCancel,
  }: {
    path: string;
    mode: 'password' | 'recovery';
    onVaultReady: () => void;
    onCancel: () => void;
  } = $props();

  let credential = $state('');
  let busy = $state(false);
  let error = $state<string | null>(null);

  async function unlock() {
    if (!credential.trim() || busy) return;
    busy = true;
    error = null;
    try {
      if (mode === 'password') {
        const status = await unlockVault(path, credential);
        if (!status.opened) throw new Error('master password required');
      } else {
        await recoverVault(path, credential);
      }
      onVaultReady();
    } catch (cause) {
      error = `${t('common.error')}: ${String(cause)}`;
    } finally {
      busy = false;
    }
  }
</script>

<div class="card glass-elevated spring" role="dialog" aria-labelledby="unlock-title">
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void unlock();
    }}
  >
    <h1 id="unlock-title">
      {mode === 'password'
        ? t('unlock.title', undefined, $locale)
        : t('unlock.recovery', undefined, $locale)}
    </h1>
    <p>
      {mode === 'password'
        ? t('unlock.password', undefined, $locale)
        : t('unlock.phrase', undefined, $locale)}
    </p>
    <p class="path">{path}</p>
    <label for="vault-credential">
      {mode === 'password'
        ? t('unlock.password', undefined, $locale)
        : t('unlock.phrase', undefined, $locale)}
    </label>
    {#if mode === 'password'}
      <input
        id="vault-credential"
        type="password"
        bind:value={credential}
        autocomplete="current-password"
      />
    {:else}
      <textarea
        id="vault-credential"
        rows="4"
        bind:value={credential}
        autocomplete="off"
        spellcheck="false"
      ></textarea>
    {/if}
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <div class="actions">
      <button type="button" disabled={busy} onclick={onCancel}
        >{t('common.cancel', undefined, $locale)}</button
      >
      <button type="submit" class="primary" disabled={busy || !credential.trim()}>
        {busy
          ? t('common.loading', undefined, $locale)
          : mode === 'password'
            ? t('unlock.unlock', undefined, $locale)
            : t('unlock.recover', undefined, $locale)}
      </button>
    </div>
  </form>
</div>

<style>
  .card {
    box-sizing: border-box;
    width: min(560px, calc(100vw - 32px));
    padding: 36px 40px;
    color: var(--glass-text);
  }
  h1 {
    margin: 0 0 12px;
    font-size: 28px;
  }
  p {
    margin: 0 0 16px;
    color: var(--glass-text-dim);
    line-height: 1.5;
  }
  .path {
    color: var(--glass-gold);
    overflow-wrap: anywhere;
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 12px;
  }
  label {
    display: block;
    margin-bottom: 8px;
    font-weight: 600;
  }
  textarea,
  input {
    box-sizing: border-box;
    width: 100%;
    padding: 12px;
    border: 1px solid var(--glass-border-strong);
    border-radius: 10px;
    background: var(--glass-input);
    color: var(--glass-text);
    font: 13px/1.6 'JetBrains Mono', ui-monospace, monospace;
    transition:
      border-color var(--motion-duration) ease,
      box-shadow var(--motion-duration) ease;
  }
  textarea {
    resize: vertical;
  }
  textarea:hover,
  input:hover {
    border-color: color-mix(in srgb, var(--glass-periwinkle) 35%, var(--glass-border-strong));
  }
  textarea:focus-visible,
  input:focus-visible {
    outline: none;
    border-color: color-mix(in srgb, var(--glass-periwinkle) 60%, transparent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--glass-periwinkle) 18%, transparent);
  }
  button:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .error {
    margin-top: 12px;
    color: var(--glass-danger);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 20px;
  }
  button {
    padding: 10px 18px;
    border: 1px solid var(--glass-border);
    border-radius: 10px;
    background: var(--glass-control-bg);
    color: var(--glass-text);
    cursor: pointer;
    font: inherit;
  }
  button:hover:not(:disabled) {
    background: var(--glass-hover-strong);
  }
  button.primary {
    border-color: transparent;
    background: var(--glass-accent);
    color: #fff;
  }
  button.primary:hover:not(:disabled) {
    filter: brightness(1.05);
  }
  button:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
</style>
