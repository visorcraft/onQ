<script lang="ts">
  import { pickVaultDir, setupNewVault, unlockVault } from '$lib/api/vault';
  import RecoveryPhrase from './RecoveryPhrase.svelte';
  import VaultUnlock from './VaultUnlock.svelte';

  let {
    onVaultReady,
    initialError = null,
    initialRecoveryPath = null,
  }: {
    onVaultReady: (arg0: string) => void; // eslint-disable-line no-unused-vars
    initialError?: string | null;
    initialRecoveryPath?: string | null;
  } = $props();

  let busy = $state(false);
  let error = $state<string | null>(null);
  let recovery = $state<string | null>(null);
  let createPath = $state<string | null>(null);
  let masterPassword = $state('');
  let confirmPassword = $state('');
  let recoverablePath = $state<string | null>(null);
  let unlockPath = $state<string | null>(null);
  let unlockMode = $state<'password' | 'recovery'>('password');

  $effect(() => {
    error = initialError;
    recoverablePath = initialRecoveryPath;
  });

  async function start() {
    busy = true;
    error = null;
    try {
      const path = await pickVaultDir();
      if (!path) return;
      createPath = path;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function create(password: string | null) {
    if (!createPath) return;
    busy = true;
    error = null;
    try {
      const result = await setupNewVault(createPath, password);
      if (result.recoveryPhrase) recovery = result.recoveryPhrase;
      else onVaultReady(createPath);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function openExisting() {
    busy = true;
    error = null;
    try {
      const path = await pickVaultDir();
      if (!path) return;
      const status = await unlockVault(path);
      if (status.opened) onVaultReady(path);
      else if (status.needsPassword) {
        unlockPath = path;
        unlockMode = 'password';
      } else if (status.needsRecovery) {
        recoverablePath = path;
        error = 'Encryption key missing from system keychain.';
      }
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  function recoverySaved() {
    recovery = null;
    onVaultReady(createPath ?? '.');
  }

  function recovered() {
    if (unlockPath) onVaultReady(unlockPath);
  }

  function recover() {
    if (!recoverablePath) return;
    unlockPath = recoverablePath;
    unlockMode = 'recovery';
  }
</script>

{#if !unlockPath}
<div class="hero glass-elevated spring" role="dialog" aria-labelledby="welcome">
  {#if createPath}
    <h1 id="welcome">Protect your vault</h1>
    <p>
      Set a master password, or let onQ create and store an encryption
      key in your system keychain.
    </p>
    <label for="master-password">Master password</label>
    <input
      id="master-password"
      type="password"
      bind:value={masterPassword}
      autocomplete="new-password"
    />
    <label for="confirm-password">Confirm master password</label>
    <input
      id="confirm-password"
      type="password"
      bind:value={confirmPassword}
      autocomplete="new-password"
    />
    {#if masterPassword && masterPassword !== confirmPassword}
      <p class="err" role="alert">Passwords do not match.</p>
    {/if}
  {:else}
    <h1 id="welcome">Welcome to onQ</h1>
    <p>Search-oriented encrypted prompt vault.</p>
  {/if}
  {#if error}
    <p class="err" role="alert">{error}</p>
  {/if}
  <div class="actions" class:create-actions={createPath}>
    {#if createPath}
      <button
        class="primary"
        disabled={busy || !masterPassword || masterPassword !== confirmPassword}
        onclick={() => void create(masterPassword)}
      >
        Create with password
      </button>
      <div class="secondary-actions">
        <button disabled={busy} onclick={() => (createPath = null)}>Back</button>
        <button disabled={busy} onclick={() => void create(null)}>
          Create without password
        </button>
      </div>
    {:else}
      <button class="primary" disabled={busy} onclick={start}>Create new vault</button>
      <button disabled={busy} onclick={openExisting}>Open existing</button>
      {#if recoverablePath}
        <button disabled={busy} onclick={recover}>Recover with recovery phrase</button>
      {/if}
    {/if}
  </div>
</div>
{/if}

{#if recovery}
  <RecoveryPhrase phrase={recovery} onAcknowledged={recoverySaved} />
{/if}
{#if unlockPath}
  <VaultUnlock
    path={unlockPath}
    mode={unlockMode}
    onVaultReady={recovered}
    onCancel={() => (unlockPath = null)}
  />
{/if}

<style>
  .hero {
    padding: 48px 64px;
    text-align: center;
    color: var(--glass-text);
    max-width: 520px;
  }
  h1 {
    font-size: 32px;
    font-weight: 600;
    margin: 0 0 12px;
  }
  p {
    color: var(--glass-text-dim);
    margin: 0 0 24px;
  }
  label {
    display: block;
    margin: 14px 0 6px;
    text-align: left;
    font-size: 14px;
    font-weight: 600;
  }
  input {
    box-sizing: border-box;
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--glass-border-strong);
    border-radius: 10px;
    background: var(--glass-input);
    color: var(--glass-text);
    font: inherit;
    transition:
      border-color var(--motion-duration) ease,
      box-shadow var(--motion-duration) ease;
  }
  input:hover {
    border-color: color-mix(in srgb, var(--glass-periwinkle) 35%, var(--glass-border-strong));
  }
  input:focus-visible {
    outline: none;
    border-color: color-mix(in srgb, var(--glass-periwinkle) 60%, transparent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--glass-periwinkle) 18%, transparent);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    justify-content: center;
  }
  .create-actions {
    flex-direction: column;
    align-items: center;
    margin-top: 24px;
  }
  .secondary-actions {
    display: flex;
    gap: 12px;
  }
  button {
    padding: 10px 20px;
    border-radius: 10px;
    border: 1px solid var(--glass-border);
    background: var(--glass-control-bg);
    color: var(--glass-text);
    cursor: pointer;
    font: inherit;
  }
  button.primary {
    background: var(--glass-accent);
    border-color: transparent;
    color: #fff;
  }
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  button:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .err {
    color: #ff7a7a;
    margin-top: 12px;
  }
</style>
