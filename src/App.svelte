<script lang="ts">
  import { onMount } from 'svelte';
  import { check } from '@tauri-apps/plugin-updater';
  import Palette from '$lib/components/Palette.svelte';
  import Editor from '$lib/components/Editor.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import SettingsPanel from '$lib/components/SettingsPanel.svelte';
  import TutorialOverlay from '$lib/components/TutorialOverlay.svelte';
  import VaultUnlock from '$lib/components/VaultUnlock.svelte';
  import { theme, setTheme, type Theme } from '$lib/stores/theme';
  import {
    checkAndStart,
    reset as replayTutorial,
    tutorialVisible,
  } from '$lib/stores/tutorial';
  import { getAppSetting } from '$lib/api/settings';
  import { readPrompt } from '$lib/api/prompts';
  import { paletteShortcut } from '$lib/shortcut';
  import { globalShortcut } from '$lib/stores/globalShortcut';
  import { openLastVault } from '$lib/api/vault';

  const shortcut = paletteShortcut();
  // Id of the prompt the user had open when the app last quit. Read from
  // the encrypted `app_state.last_opened_prompt` column once the vault is
  // ready and, if it still resolves, pre-loaded into the editor so the user
  // returns to where they left off.
  let lastOpenedId = $state<string | null>(null);
  let hasVault = $state(false);
  let checkingVault = $state(true);
  let vaultError = $state<string | null>(null);
  let passwordPath = $state<string | null>(null);
  let recoveryPath = $state<string | null>(null);
  let tutorialChecked = false;
  // M6.10: settings drawer state. Bound to the SettingsPanel modal so
  // the panel can dismiss itself by flipping `open = false`.
  let settingsOpen = $state(false);
  let checkingForUpdates = $state(false);
  let updateStatus = $state<string | null>(null);

  async function checkForUpdates(manual = false) {
    if (checkingForUpdates) return;
    checkingForUpdates = true;
    if (manual) updateStatus = 'Checking for updates…';

    try {
      const update = await check();
      if (update) {
        updateStatus = `onQ ${update.version} is available`;
        await update.close();
      } else if (manual) {
        updateStatus = 'onQ is up to date';
      }
    } catch (error) {
      if (manual) updateStatus = `Unable to check for updates: ${String(error)}`;
    } finally {
      checkingForUpdates = false;
    }
  }

  onMount(async () => {
    try {
      const status = await openLastVault();
      if (status.opened) onVaultReady();
      else if (status.needsPassword && status.path) passwordPath = status.path;
      else if (status.needsRecovery && status.path) {
        recoveryPath = status.path;
        vaultError = 'Encryption key missing from system keychain.';
      }
    } catch (error) {
      vaultError = `Could not open last vault: ${String(error)}`;
    } finally {
      checkingVault = false;
    }
    void checkForUpdates();
  });

  async function restoreLastOpened() {
    try {
      const id = await getAppSetting('last_opened_prompt');
      if (!id) return;
      // `readPrompt` throws if the id was deleted between sessions — the
      // catch below swallows it so a stale id never prevents the app
      // from launching.
      await readPrompt(id);
      lastOpenedId = id;
    } catch {
      lastOpenedId = null;
    }
  }

  $effect(() => {
    if (!hasVault || tutorialChecked) return;
    tutorialChecked = true;
    void restoreLastOpened();
    void checkAndStart(hasVault).catch(() => undefined);
  });

  function onVaultReady() {
    passwordPath = null;
    recoveryPath = null;
    hasVault = true;
  }

  function toggleTheme() {
    const next: Theme = $theme === 'dark' ? 'light' : 'dark';
    void setTheme(next);
  }

  function openSettings() {
    settingsOpen = true;
  }

  function closeEditor() {
    lastOpenedId = null;
  }
</script>

<main>
  <div class="app-controls">
    <button
      type="button"
      class="update-button glass"
      aria-label="Check for updates"
      onclick={() => void checkForUpdates(true)}
      disabled={checkingForUpdates}
    >
      {checkingForUpdates ? 'Checking…' : 'Check for updates'}
    </button>
    {#if hasVault}
      <button
        type="button"
        class="icon-button help-button glass"
        aria-label="Replay tutorial"
        title="Replay tutorial"
        onclick={replayTutorial}
      >
        ?
      </button>
      <button
        type="button"
        class="icon-button settings-button glass"
        aria-label="Open settings"
        title="Settings"
        onclick={openSettings}
      >
        ⚙
      </button>
      <button
        type="button"
        class="icon-button theme-toggle glass"
        aria-label="Toggle theme"
        title="Toggle theme"
        onclick={toggleTheme}
      >
        {$theme === 'dark' ? '☀️' : '🌙'}
      </button>
    {/if}
  </div>
  {#if updateStatus}
    <p class="update-status" role="status">{updateStatus}</p>
  {/if}
  {#if checkingVault}
    <p role="status">Opening last vault…</p>
  {:else if passwordPath}
    <VaultUnlock
      path={passwordPath}
      mode="password"
      {onVaultReady}
      onCancel={() => (passwordPath = null)}
    />
  {:else if hasVault}
    <div class="hero glass spring">
      <h1>onQ</h1>
      <p>Press <kbd>{$globalShortcut || shortcut}</kbd> to begin</p>
    </div>
    <Palette />
    {#if lastOpenedId}
      <Editor id={lastOpenedId} onClose={closeEditor} />
    {/if}
    {#if $tutorialVisible}
      <TutorialOverlay />
    {/if}
    <SettingsPanel bind:open={settingsOpen} />
  {:else}
    <EmptyState
      {onVaultReady}
      initialError={vaultError}
      initialRecoveryPath={recoveryPath}
    />
  {/if}
</main>

<style>
  main {
    box-sizing: border-box;
    position: relative;
    min-height: 100vh;
    display: grid;
    place-items: center;
    padding: 24px;
  }
  .hero {
    padding: 64px 80px;
    text-align: center;
    color: var(--glass-text);
  }
  h1 {
    font-size: 40px;
    font-weight: 600;
    letter-spacing: -0.02em;
    margin: 0 0 12px;
  }
  p {
    color: var(--glass-text-dim);
    margin: 0;
  }
  kbd {
    font-family: 'JetBrains Mono', monospace;
    background: rgba(127, 127, 127, 0.15);
    padding: 2px 8px;
    border-radius: 6px;
  }
  .app-controls {
    position: absolute;
    top: 24px;
    right: 24px;
    display: flex;
    gap: 8px;
  }
  .update-button {
    height: 40px;
    padding: 0 14px;
    border: 1px solid var(--glass-border);
    background: rgba(127, 127, 127, 0.08);
    color: var(--glass-text);
    cursor: pointer;
    font: inherit;
  }
  .update-button:hover:not(:disabled) {
    background: rgba(127, 127, 127, 0.16);
  }
  .update-button:disabled {
    cursor: wait;
    opacity: 0.65;
  }
  .update-button:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .update-status {
    position: absolute;
    top: 72px;
    right: 24px;
    margin: 0;
    color: var(--glass-text-dim);
    font-size: 14px;
  }
  .icon-button {
    width: 40px;
    height: 40px;
    display: grid;
    place-items: center;
    line-height: 1;
    color: var(--glass-text);
    cursor: pointer;
    border: 1px solid var(--glass-border);
    background: rgba(127, 127, 127, 0.08);
  }
  .help-button {
    font-size: 20px;
    font-weight: 700;
  }
  .theme-toggle {
    font-size: 18px;
  }
  .settings-button {
    font-size: 18px;
  }
  .icon-button:hover {
    background: rgba(127, 127, 127, 0.16);
  }
  .icon-button:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
</style>
