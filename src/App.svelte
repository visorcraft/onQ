<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { check } from '@tauri-apps/plugin-updater';
  import Palette from '$lib/components/Palette.svelte';
  import Editor from '$lib/components/Editor.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import SettingsPage from '$lib/components/SettingsPage.svelte';
  import LibraryPage from '$lib/components/LibraryPage.svelte';
  import AboutPage from '$lib/components/about/AboutPage.svelte';
  import LicensesPage from '$lib/components/about/LicensesPage.svelte';
  import CreditsPage from '$lib/components/about/CreditsPage.svelte';
  import TutorialOverlay from '$lib/components/TutorialOverlay.svelte';
  import VaultUnlock from '$lib/components/VaultUnlock.svelte';
  import { theme, setTheme, type Theme } from '$lib/stores/theme';
  import {
    checkAndStart,
    tutorialVisible,
  } from '$lib/stores/tutorial';
  import { paletteShortcut } from '$lib/shortcut';
  import { globalShortcut } from '$lib/stores/globalShortcut';
  import { openLastVault } from '$lib/api/vault';
  import { appView, navigate, type AppView } from '$lib/stores/navigation';
  import { version as appVersion } from '../package.json';

  const shortcut = paletteShortcut();
  const STATUS_CLEAR_MS = 5_000;
  let lastOpenedId = $state<string | null>(null);
  /** Bumped when the editor closes so Library reloads list data. */
  let libraryEpoch = $state(0);
  let hasVault = $state(false);
  let checkingVault = $state(true);
  let vaultError = $state<string | null>(null);
  let passwordPath = $state<string | null>(null);
  let recoveryPath = $state<string | null>(null);
  let tutorialChecked = false;
  let checkingForUpdates = $state(false);
  let updateStatus = $state<string | null>(null);
  let updateStatusTimer: ReturnType<typeof setTimeout> | undefined;

  function clearUpdateStatusTimer() {
    if (updateStatusTimer !== undefined) {
      clearTimeout(updateStatusTimer);
      updateStatusTimer = undefined;
    }
  }

  function setUpdateStatus(message: string | null, autoClearMs?: number) {
    clearUpdateStatusTimer();
    updateStatus = message;
    if (message && autoClearMs !== undefined) {
      updateStatusTimer = setTimeout(() => {
        updateStatus = null;
        updateStatusTimer = undefined;
      }, autoClearMs);
    }
  }

  async function checkForUpdates(manual = false) {
    if (checkingForUpdates) return;
    checkingForUpdates = true;
    if (manual) setUpdateStatus('Checking for updates…');

    try {
      const update = await check();
      if (update) {
        setUpdateStatus(`onQ ${update.version} is available`);
        await update.close();
      } else if (manual) {
        setUpdateStatus('onQ is up to date', STATUS_CLEAR_MS);
      } else {
        setUpdateStatus(null);
      }
    } catch (error) {
      if (manual) {
        setUpdateStatus(`Unable to check for updates: ${String(error)}`);
      }
    } finally {
      checkingForUpdates = false;
    }
  }

  onMount(() => {
    // Updater must not wait on vault open / keychain prompts — it has no
    // dependency on vault state and should work from the empty-state screen.
    void checkForUpdates();

    void (async () => {
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
    })();
  });

  onDestroy(() => {
    clearUpdateStatusTimer();
  });

  $effect(() => {
    if (!hasVault || tutorialChecked) return;
    tutorialChecked = true;
    // Do not auto-open the editor (or palette) after unlock — land on home.
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

  function go(view: AppView) {
    navigate(view);
  }

  function closeEditor() {
    lastOpenedId = null;
    libraryEpoch += 1;
  }

  const onPage = $derived($appView !== 'home');
</script>

<main class:page-mode={onPage}>
  {#if $appView === 'home'}
    <div class="app-controls">
      <button
        type="button"
        class="icon-button help-button glass"
        aria-label="About onQ"
        title="About onQ"
        onclick={() => go('about')}
      >
        ?
      </button>
      {#if hasVault}
        <button
          type="button"
          class="icon-button library-button glass"
          aria-label="Open library"
          title="Library"
          onclick={() => go('library')}
        >
          ☰
        </button>
      {/if}
      <button
        type="button"
        class="icon-button settings-button glass"
        aria-label="Open settings"
        title="Settings"
        onclick={() => go('settings')}
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
    </div>
    {#if updateStatus}
      <p class="update-status" role="status">{updateStatus}</p>
    {/if}
  {/if}

  <button
    type="button"
    class="app-version"
    aria-label="App version, check for updates"
    title="Check for updates"
    onclick={() => void checkForUpdates(true)}
    disabled={checkingForUpdates}
  >
    v{appVersion}
  </button>

  {#if $appView === 'library'}
    <LibraryPage
      {libraryEpoch}
      onBack={() => {
        lastOpenedId = null;
        go('home');
      }}
      onOpenPrompt={(id) => {
        lastOpenedId = id;
      }}
    />
    {#if lastOpenedId}
      {#key lastOpenedId}
        <Editor id={lastOpenedId} onClose={closeEditor} />
      {/key}
    {/if}
  {:else if $appView === 'settings'}
    <SettingsPage
      onBack={() => go('home')}
      onOpenAbout={() => go('about')}
      onOpenLibrary={hasVault ? () => go('library') : undefined}
    />
  {:else if $appView === 'about'}
    <AboutPage
      onBack={() => go('home')}
      onLicenses={() => go('licenses')}
      onCredits={() => go('credits')}
    />
  {:else if $appView === 'licenses'}
    <LicensesPage onBack={() => go('about')} />
  {:else if $appView === 'credits'}
    <CreditsPage onBack={() => go('about')} />
  {:else if checkingVault}
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
      {#key lastOpenedId}
        <Editor id={lastOpenedId} onClose={closeEditor} />
      {/key}
    {/if}
    {#if $tutorialVisible}
      <TutorialOverlay />
    {/if}
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
  main.page-mode {
    display: block;
    place-items: unset;
    align-content: start;
    padding: 0;
    overflow: auto;
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
    z-index: 10;
  }
  .update-status {
    position: absolute;
    top: 72px;
    right: 24px;
    margin: 0;
    color: var(--glass-text-dim);
    font-size: 14px;
    z-index: 10;
  }
  .app-version {
    position: fixed;
    left: 16px;
    bottom: 12px;
    margin: 0;
    z-index: 20;
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.02em;
    color: var(--glass-text-dim);
    opacity: 0.75;
    border: 0;
    background: transparent;
    cursor: pointer;
    padding: 4px 6px;
    border-radius: 6px;
    font: inherit;
  }
  .app-version:hover:not(:disabled) {
    opacity: 1;
    color: var(--glass-text);
    background: rgba(127, 127, 127, 0.12);
  }
  .app-version:disabled {
    cursor: wait;
  }
  .app-version:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
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
  .library-button {
    font-size: 18px;
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
