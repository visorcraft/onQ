<script lang="ts">
  import { onMount } from 'svelte';
  import { getVaultAuthMode, retrieveVaultKey } from '$lib/api/vault';
  import {
    captureGlobalShortcut,
    globalShortcut,
    globalShortcutBackend,
    setGlobalShortcut,
    shortcutFromKeyboardEvent,
  } from '$lib/stores/globalShortcut';
  import { metaModifierLabel } from '$lib/shortcut';
  import {
    betaChannel,
    embeddingQuant,
    loadBetaChannel,
    loadEmbeddingQuant,
    rebuildingIndex,
    setBetaChannel,
    setEmbeddingQuant,
    type EmbeddingQuant,
  } from '$lib/stores/settings';
  import { theme, setTheme, type Theme } from '$lib/stores/theme';

  let {
    onBack,
    onOpenLibrary,
  }: {
    onBack: () => void;
    onOpenLibrary?: () => void;
  } = $props();

  type SectionId = 'general' | 'search' | 'vault' | 'updates';

  const metaKeyLabel = metaModifierLabel();
  const sections: { id: SectionId; label: string }[] = [
    { id: 'general', label: 'General' },
    { id: 'search', label: 'Search' },
    { id: 'vault', label: 'Vault & security' },
    { id: 'updates', label: 'Updates' },
  ];

  let active = $state<SectionId>('general');
  let errorMessage = $state<string | null>(null);
  let recordingShortcut = $state(false);
  let shortcutError = $state<string | null>(null);
  let authMode = $state<'keychain' | 'password' | null>(null);
  let recoveryPhrase = $state('');
  let encryptionKey = $state<string | null>(null);
  let keyError = $state<string | null>(null);
  let retrievingKey = $state(false);
  let pendingQuant = $state<EmbeddingQuant>('binary');
  let lastSynced = $state<EmbeddingQuant | null>(null);
  let pendingBeta = $state(false);
  let lastSyncedBeta = $state<boolean | null>(null);

  $effect(() => {
    const store = $embeddingQuant;
    if (lastSynced !== store) {
      pendingQuant = store;
      lastSynced = store;
    }
  });

  $effect(() => {
    const store = $betaChannel;
    if (lastSyncedBeta !== store) {
      pendingBeta = store;
      lastSyncedBeta = store;
    }
  });

  onMount(() => {
    void loadEmbeddingQuant().catch(() => undefined);
    void loadBetaChannel().catch(() => undefined);
    void getVaultAuthMode()
      .then((mode) => (authMode = mode))
      .catch(() => undefined);
  });

  async function recordShortcut(event: KeyboardEvent) {
    event.preventDefault();
    event.stopPropagation();
    if (!recordingShortcut) return;
    if ($globalShortcutBackend === 'linux-input') return;
    if (event.key === 'Escape') {
      recordingShortcut = false;
      return;
    }
    const shortcut = shortcutFromKeyboardEvent(event);
    if (!shortcut) return;
    recordingShortcut = false;
    shortcutError = null;
    try {
      await setGlobalShortcut(shortcut);
    } catch (error) {
      shortcutError = error instanceof Error ? error.message : String(error);
    }
  }

  async function startShortcutSetup() {
    shortcutError = null;
    recordingShortcut = true;
    if ($globalShortcutBackend !== 'linux-input') return;
    try {
      await captureGlobalShortcut();
    } catch (error) {
      shortcutError = error instanceof Error ? error.message : String(error);
    } finally {
      recordingShortcut = false;
    }
  }

  async function pickQuant(next: EmbeddingQuant) {
    if (next === $embeddingQuant) return;
    errorMessage = null;
    const prev = $embeddingQuant;
    try {
      await setEmbeddingQuant(next);
    } catch (e) {
      pendingQuant = prev;
      errorMessage = e instanceof Error ? e.message : String(e);
    }
  }

  async function flipBeta(next: boolean) {
    if (next === $betaChannel) return;
    errorMessage = null;
    const prev = $betaChannel;
    try {
      await setBetaChannel(next);
    } catch (e) {
      pendingBeta = prev;
      errorMessage = e instanceof Error ? e.message : String(e);
    }
  }

  async function showEncryptionKey() {
    if (!recoveryPhrase.trim() || retrievingKey) return;
    retrievingKey = true;
    keyError = null;
    try {
      encryptionKey = await retrieveVaultKey(recoveryPhrase);
      recoveryPhrase = '';
    } catch (error) {
      keyError = error instanceof Error ? error.message : String(error);
    } finally {
      retrievingKey = false;
    }
  }

  function toggleTheme() {
    const next: Theme = $theme === 'dark' ? 'light' : 'dark';
    void setTheme(next);
  }

</script>

<div class="settings-page">
  <header class="settings-top">
    <div>
      <h1>Settings</h1>
      <p class="sub">Search-first vault controls, organization, and power tools.</p>
    </div>
  </header>

  <div class="settings-layout">
    <nav class="settings-nav" aria-label="Settings sections">
      {#each sections as s (s.id)}
        <button
          type="button"
          class="nav-item"
          class:active={active === s.id}
          onclick={() => (active = s.id)}
        >
          {s.label}
        </button>
      {/each}
    </nav>

    <div class="settings-main">
      {#if active === 'general'}
        <section class="panel" aria-labelledby="shortcut-heading">
          <h2 id="shortcut-heading">Show onQ</h2>
          <p class="help">
            Global shortcut restores onQ from the tray and opens the prompt
            palette. Default: Win+Q / Meta+Q / ⌘+Q.
          </p>
          <button
            type="button"
            class:recording={recordingShortcut}
            class="control-btn"
            onclick={() => void startShortcutSetup()}
            onkeydown={recordShortcut}
          >
            {#if recordingShortcut}
              Press shortcut…
            {:else if $globalShortcut}
              {$globalShortcut}
            {:else}
              Record shortcut
            {/if}
          </button>
          {#if recordingShortcut}
            <p class="hint">Press Ctrl, Alt, or {metaKeyLabel} plus another key. Escape cancels.</p>
          {/if}
          {#if shortcutError}
            <p class="error" role="alert">{shortcutError}</p>
          {/if}
        </section>

        <section class="panel">
          <h2>Appearance</h2>
          <p class="help">Theme applies immediately across the shell.</p>
          <button type="button" class="control-btn" onclick={toggleTheme}>
            Theme: {$theme === 'dark' ? 'Dark' : 'Light'} (click to toggle)
          </button>
        </section>

        <section class="panel">
          <h2>Library</h2>
          <p class="help">
            Projects, smart folders, favorites, and prompt browsing live in the
            Library — not in Settings.
          </p>
          {#if onOpenLibrary}
            <button type="button" class="control-btn" onclick={onOpenLibrary}
              >Open Library…</button
            >
          {:else}
            <p class="hint">Use the ☰ button on the home screen.</p>
          {/if}
        </section>
      {:else if active === 'search'}
        <section class="panel" aria-labelledby="embedding-heading">
          <h2 id="embedding-heading">Embedding quantization</h2>
          <p class="help">
            Trade-off between recall and search speed on the
            <code>prompts.embedding</code> ANN index.
          </p>
          <fieldset class="radio-group" disabled={$rebuildingIndex}>
            <legend class="visually-hidden">Embedding quantization mode</legend>
            <label class="radio-card" class:selected={pendingQuant === 'binary'}>
              <input
                type="radio"
                name="embedding-quant"
                value="binary"
                bind:group={pendingQuant}
                onchange={() => pickQuant('binary')}
              />
              <span class="radio-label">Binary (default)</span>
              <span class="radio-desc">
                Binary HNSW candidates + exact cosine rerank. Fast; low memory.
              </span>
            </label>
            <label class="radio-card" class:selected={pendingQuant === 'dense'}>
              <input
                type="radio"
                name="embedding-quant"
                value="dense"
                bind:group={pendingQuant}
                onchange={() => pickQuant('dense')}
              />
              <span class="radio-label">Dense</span>
              <span class="radio-desc">
                Full f32 cosine scan. Higher recall; slower on large vaults.
              </span>
            </label>
          </fieldset>
          {#if $rebuildingIndex}
            <p class="hint" role="status">Updating embedding index…</p>
          {/if}
        </section>
      {:else if active === 'vault'}
        {#if authMode === 'keychain'}
          <section class="panel">
            <h2>Encryption key</h2>
            <p class="help">
              This vault uses a key stored in your system keychain. Enter its
              recovery phrase only when you need to display that key.
            </p>
            {#if encryptionKey}
              <textarea readonly rows="3" value={encryptionKey}></textarea>
            {:else}
              <textarea
                rows="3"
                bind:value={recoveryPhrase}
                placeholder="24-word recovery phrase"
                autocomplete="off"
                spellcheck="false"
              ></textarea>
              <button
                type="button"
                class="control-btn"
                disabled={retrievingKey || !recoveryPhrase.trim()}
                onclick={() => void showEncryptionKey()}
              >
                {retrievingKey ? 'Checking…' : 'Show encryption key'}
              </button>
            {/if}
            {#if keyError}<p class="error" role="alert">{keyError}</p>{/if}
          </section>
        {:else}
          <section class="panel">
            <h2>Vault security</h2>
            <p class="help">
              Password-protected vault. Unlock when prompted; auto-lock policies
              apply based on your vault settings.
            </p>
            <p class="hint">Auth mode: {authMode ?? 'unknown'}</p>
          </section>
        {/if}
      {:else if active === 'updates'}
        <section class="panel">
          <h2>Beta channel</h2>
          <p class="help">
            Receive pre-release auto-updates earlier. Production
            <code>latest.json</code> remains the default feed until beta
            routing ships.
          </p>
          <label class="toggle-row">
            <input
              type="checkbox"
              bind:checked={pendingBeta}
              onchange={(event) => flipBeta(event.currentTarget.checked)}
            />
            <span class="toggle-label">Opt in to beta releases</span>
          </label>
        </section>
      {/if}

      {#if errorMessage}
        <p class="error" role="alert">{errorMessage}</p>
      {/if}
    </div>
  </div>

  <button type="button" class="page-back" onclick={onBack}>← Back</button>
</div>

<style>
  .settings-page {
    box-sizing: border-box;
    width: 100%;
    margin: 0;
    padding: 20px 24px 56px;
    color: var(--glass-text);
  }
  .settings-top {
    margin-bottom: 20px;
  }
  h1 {
    margin: 0 0 4px;
    font-size: 28px;
    font-weight: 700;
  }
  h2 {
    margin: 0 0 8px;
    font-size: 16px;
  }
  .sub,
  .help {
    margin: 0 0 12px;
    color: var(--glass-text-dim);
    font-size: 13px;
    line-height: 1.45;
  }
  .help code {
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 12px;
  }
  .settings-layout {
    display: grid;
    grid-template-columns: 200px minmax(0, 1fr);
    gap: 18px;
    align-items: start;
  }
  @media (max-width: 800px) {
    .settings-layout {
      grid-template-columns: 1fr;
    }
  }
  .settings-nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px;
    border-radius: 14px;
    border: 1px solid var(--glass-border);
    background: var(--glass-inset);
    position: sticky;
    top: 16px;
  }
  .nav-item {
    appearance: none;
    border: 0;
    background: transparent;
    color: var(--glass-text-dim);
    text-align: left;
    padding: 10px 12px;
    border-radius: 10px;
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .nav-item.active {
    color: var(--glass-selected-fg);
    background: var(--glass-selected-bg);
  }
  .nav-item:hover {
    background: var(--glass-hover);
  }
  .settings-main {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .panel {
    padding: 18px;
    border-radius: 14px;
    border: 1px solid var(--glass-border);
    background: var(--glass-panel);
  }
  .control-btn,
  .page-back {
    appearance: none;
    border: 1px solid var(--glass-border);
    background: var(--glass-control-bg);
    color: var(--glass-text);
    border-radius: 10px;
    padding: 10px 14px;
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .control-btn.recording {
    border-color: var(--glass-periwinkle);
  }
  .control-btn:hover,
  .page-back:hover {
    background: var(--glass-hover-strong);
  }
  .control-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .page-back {
    position: fixed;
    left: 16px;
    bottom: 12px;
    z-index: 20;
    border-radius: 999px;
    padding: 8px 14px;
    font-size: 13px;
    opacity: 0.9;
  }
  .page-back:hover {
    opacity: 1;
  }
  .page-back:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .hint {
    margin: 8px 0 0;
    font-size: 12px;
    color: var(--glass-text-dim);
  }
  .error {
    margin: 8px 0 0;
    color: #c04040;
    font-size: 13px;
  }
  :global(:root.dark) .error {
    color: #ffb4b4;
  }
  .radio-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 0;
    padding: 0;
    border: 0;
  }
  .radio-card {
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-rows: auto auto;
    column-gap: 10px;
    padding: 10px 12px;
    border: 1px solid var(--glass-border);
    border-radius: 10px;
    cursor: pointer;
  }
  .radio-card.selected {
    border-color: var(--glass-periwinkle);
    background: var(--glass-selected-bg);
  }
  .radio-card input {
    grid-row: 1 / span 2;
    margin-top: 3px;
  }
  .radio-label {
    font-weight: 600;
    font-size: 14px;
  }
  .radio-desc {
    font-size: 12px;
    color: var(--glass-text-dim);
  }
  .toggle-row {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 10px 12px;
    border: 1px solid var(--glass-border);
    border-radius: 10px;
    cursor: pointer;
    font-size: 13px;
    color: var(--glass-text);
  }
  .toggle-label {
    font-size: 13px;
    line-height: 1.4;
    color: var(--glass-text);
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    border-radius: 10px;
    border: 1px solid var(--glass-border);
    background: var(--glass-input);
    color: var(--glass-text);
    padding: 10px 12px;
    font: inherit;
    margin-bottom: 8px;
  }
  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    border: 0;
  }
</style>
