<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
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
    loadMinimizeOnCopy,
    minimizeOnCopy,
    rebuildingIndex,
    setBetaChannel,
    setEmbeddingQuant,
    setMinimizeOnCopy,
    type EmbeddingQuant,
  } from '$lib/stores/settings';
  import {
    ensureSearchModel,
    getSearchStatus,
    type SearchStatus,
  } from '$lib/api/search';
  import {
    checkForAppUpdates,
    formatUpdateStatus,
    type UpdateCheckOutcome,
  } from '$lib/api/updates';
  import { theme, setTheme, type Theme } from '$lib/stores/theme';
  import BackupsSection from '$lib/components/settings/BackupsSection.svelte';
  import PluginsSection from '$lib/components/settings/PluginsSection.svelte';
  import type { ImportBackupResult } from '$lib/api/backup';
  import { getAppSetting, setAppSetting } from '$lib/api/settings';
  import { getAutoLockPolicy, setAutoLockPolicy } from '$lib/api/session';
  import {
    exportPrompts,
    importPrompts,
    pickExportDir,
    pickImportPath,
  } from '$lib/api/importExport';
  import { backupShouldRemind } from '$lib/api/backupRemind';

  type SectionId = 'general' | 'search' | 'vault' | 'backups' | 'plugins' | 'updates';

  const metaKeyLabel = metaModifierLabel();
  const sections: { id: SectionId; label: string; hint: string }[] = [
    { id: 'general', label: 'General', hint: 'Shortcut & theme' },
    { id: 'search', label: 'Search', hint: 'Model & index' },
    { id: 'vault', label: 'Vault', hint: 'Security keys' },
    { id: 'backups', label: 'Backups', hint: 'Export & restore' },
    { id: 'plugins', label: 'Plugins', hint: 'Install & manage' },
    { id: 'updates', label: 'Updates', hint: 'Release channel' },
  ];

  let {
    onVaultClosed,
  }: {
    /** After destructive import: vault session closed, show unlock. */
    onVaultClosed?: (importResult: ImportBackupResult) => void; // eslint-disable-line no-unused-vars
  } = $props();

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
  let pendingMinimize = $state(false);
  let lastSyncedMinimize = $state<boolean | null>(null);
  let searchStatus = $state<SearchStatus | null>(null);
  let loadingModel = $state(false);
  let searchStatusError = $state<string | null>(null);
  let checkingForUpdates = $state(false);
  let updateStatus = $state<string | null>(null);
  let updateStatusTimer: ReturnType<typeof setTimeout> | undefined;
  let recencyDays = $state('30');
  let historyDays = $state('30');
  let autoLockPolicy = $state('lock_on_quit');
  let autoLockIdleMinutes = $state('15');
  let importStatus = $state<string | null>(null);
  let backupRemind = $state(false);

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

  async function runUpdateCheck() {
    if (checkingForUpdates) return;
    checkingForUpdates = true;
    setUpdateStatus('Checking for updates…');
    try {
      const outcome: UpdateCheckOutcome = await checkForAppUpdates(true);
      const formatted = formatUpdateStatus(outcome);
      if (formatted) {
        setUpdateStatus(formatted, outcome.kind === 'up_to_date' ? 5_000 : undefined);
      }
    } finally {
      checkingForUpdates = false;
    }
  }

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

  $effect(() => {
    const store = $minimizeOnCopy;
    if (lastSyncedMinimize !== store) {
      pendingMinimize = store;
      lastSyncedMinimize = store;
    }
  });

  async function refreshSearchStatus() {
    searchStatusError = null;
    try {
      searchStatus = await getSearchStatus();
    } catch (e) {
      searchStatus = null;
      searchStatusError = e instanceof Error ? e.message : String(e);
    }
  }

  async function loadSearchModel() {
    loadingModel = true;
    searchStatusError = null;
    errorMessage = null;
    try {
      searchStatus = await ensureSearchModel();
    } catch (e) {
      searchStatusError = e instanceof Error ? e.message : String(e);
    } finally {
      loadingModel = false;
    }
  }

  onMount(() => {
    void loadEmbeddingQuant().catch(() => undefined);
    void loadBetaChannel().catch(() => undefined);
    void loadMinimizeOnCopy().catch(() => undefined);
    void getVaultAuthMode()
      .then((mode) => (authMode = mode))
      .catch(() => undefined);
    void refreshSearchStatus();
    void getAppSetting('search_recency_days')
      .then((v) => {
        if (v) recencyDays = v;
      })
      .catch(() => undefined);
    void getAppSetting('history_retention_days')
      .then((v) => {
        if (v) historyDays = v;
      })
      .catch(() => undefined);
    void getAutoLockPolicy()
      .then((p) => {
        autoLockPolicy = p.startsWith('idle_timeout:') ? 'idle' : p;
        if (p.startsWith('idle_timeout:')) {
          const secs = Number(p.slice('idle_timeout:'.length));
          if (Number.isFinite(secs) && secs > 0) autoLockIdleMinutes = String(Math.round(secs / 60));
        }
      })
      .catch(() => undefined);
    void backupShouldRemind()
      .then((v) => {
        backupRemind = v;
      })
      .catch(() => undefined);
  });

  async function saveRecency() {
    await setAppSetting('search_recency_days', recencyDays.trim() || '30');
  }

  async function saveHistoryRetention() {
    await setAppSetting('history_retention_days', historyDays.trim() || '30');
  }

  async function saveAutoLock() {
    if (autoLockPolicy === 'idle') {
      const mins = Math.max(1, Number(autoLockIdleMinutes) || 15);
      await setAutoLockPolicy(`idle_timeout:${mins * 60}`);
    } else {
      await setAutoLockPolicy(autoLockPolicy);
    }
  }

  async function runImport() {
    importStatus = null;
    try {
      const path = await pickImportPath();
      if (!path) return;
      const report = await importPrompts(path, 'auto', 'skip');
      importStatus = `Imported ${report.created}, skipped ${report.skipped}${
        report.errors.length ? `; ${report.errors.length} errors` : ''
      }.`;
    } catch (e) {
      importStatus = e instanceof Error ? e.message : String(e);
    }
  }

  async function runExport() {
    importStatus = null;
    try {
      const dest = await pickExportDir();
      if (!dest) return;
      const report = await exportPrompts({ dest });
      importStatus = `Exported ${report.exported} prompts (${report.skipped} skipped).`;
    } catch (e) {
      importStatus = e instanceof Error ? e.message : String(e);
    }
  }

  onDestroy(() => {
    clearUpdateStatusTimer();
  });

  $effect(() => {
    if (active === 'search') {
      void refreshSearchStatus();
    }
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

  async function flipMinimizeOnCopy(next: boolean) {
    if (next === $minimizeOnCopy) return;
    errorMessage = null;
    const prev = $minimizeOnCopy;
    try {
      await setMinimizeOnCopy(next);
    } catch (e) {
      pendingMinimize = prev;
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

  const activeSection = $derived(sections.find((s) => s.id === active) ?? sections[0]);
</script>

<div class="settings-page">
  <div class="page-glow" aria-hidden="true"></div>

  <header class="settings-top">
    <p class="eyebrow">Preferences</p>
    <h1>Settings</h1>
    <p class="sub">Search-first vault controls, organization, and power tools.</p>
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
          <span class="nav-label">{s.label}</span>
          <span class="nav-hint">{s.hint}</span>
        </button>
      {/each}
    </nav>

    <div class="settings-main">
      <div class="section-head">
        <h2>{activeSection.label}</h2>
        <p class="section-sub">{activeSection.hint}</p>
      </div>

      {#if active === 'general'}
        <section class="panel" aria-labelledby="shortcut-heading">
          <div class="panel-head">
            <h3 id="shortcut-heading">Show onQ</h3>
            <p class="help">
              Global shortcut restores onQ from the tray and opens the prompt
              palette. Default: Win+Q / Meta+Q / ⌘+Q.
            </p>
          </div>
          <button
            type="button"
            class:recording={recordingShortcut}
            class="control-btn shortcut-btn"
            onclick={() => void startShortcutSetup()}
            onkeydown={recordShortcut}
          >
            {#if recordingShortcut}
              <span class="rec-dot" aria-hidden="true"></span>
              Press shortcut…
            {:else if $globalShortcut}
              <kbd>{$globalShortcut}</kbd>
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
          <div class="panel-head">
            <h3>Appearance</h3>
            <p class="help">Theme applies immediately across the shell.</p>
          </div>
          <div class="theme-row">
            <button
              type="button"
              class="theme-card"
              class:selected={$theme === 'dark'}
              onclick={() => $theme !== 'dark' && toggleTheme()}
            >
              <span class="theme-swatch dark" aria-hidden="true"></span>
              <span class="theme-label">Dark</span>
            </button>
            <button
              type="button"
              class="theme-card"
              class:selected={$theme === 'light'}
              onclick={() => $theme !== 'light' && toggleTheme()}
            >
              <span class="theme-swatch light" aria-hidden="true"></span>
              <span class="theme-label">Light</span>
            </button>
          </div>
        </section>

        <section class="panel" aria-labelledby="palette-heading">
          <div class="panel-head">
            <h3 id="palette-heading">Palette</h3>
            <p class="help">
              Behaviour of the global command palette when you click a search
              result or one of the Recent items.
            </p>
          </div>
          <label class="toggle-row">
            <span class="toggle-copy">
              <span class="toggle-label">Automatically minimize after clicking search result</span>
              <span class="toggle-desc">Hide onQ to the system tray once a prompt is on your clipboard</span>
            </span>
            <span class="switch" class:on={pendingMinimize}>
              <input
                type="checkbox"
                bind:checked={pendingMinimize}
                onchange={(event) => flipMinimizeOnCopy(event.currentTarget.checked)}
              />
              <span class="switch-track" aria-hidden="true">
                <span class="switch-thumb"></span>
              </span>
            </span>
          </label>
        </section>

      {:else if active === 'search'}
        <section class="panel" aria-labelledby="search-status-heading">
          <div class="panel-head">
            <h3 id="search-status-heading">How search works</h3>
            <p class="help">
              Free-text search ranks prompts with hybrid retrieval: keyword
              matching over <strong>title, tags, project, and body</strong>,
              fused with semantic ANN when the embedding model is loaded.
            </p>
          </div>
          <ul class="status-list">
            <li>
              <span class="status-k">Keyword index</span>
              <span class="status-v ok">On</span>
              <span class="status-note">Includes titles &amp; tags</span>
            </li>
            <li>
              <span class="status-k">Embedding model</span>
              <span class="status-v" class:ok={searchStatus?.embedderLoaded} class:warn={!searchStatus?.embedderLoaded}>
                {#if searchStatus?.embedderLoaded}
                  Loaded
                {:else if searchStatus?.modelCached}
                  On disk (not loaded)
                {:else}
                  Not installed
                {/if}
              </span>
              <span class="status-note mono">{searchStatus?.modelId ?? 'sentence-transformers/all-MiniLM-L6-v2'}</span>
            </li>
            <li>
              <span class="status-k">Semantic path</span>
              <span class="status-v" class:ok={!searchStatus?.sparseOnly} class:warn={searchStatus?.sparseOnly}>
                {#if searchStatus?.sparseOnly}
                  Sparse only (no ANN)
                {:else if searchStatus?.embeddingQuant === 'dense' && searchStatus.denseReadiness === 'ready'}
                  Dense ANN live
                {:else if searchStatus?.embeddingQuant === 'dense' && searchStatus.denseReadiness === 'pending'}
                  Dense pending (exact cosine)
                {:else}
                  Binary ANN + rerank
                {/if}
              </span>
            </li>
          </ul>
          {#if searchStatusError}
            <p class="error" role="alert">{searchStatusError}</p>
          {/if}
          <div class="row-actions">
            <button
              type="button"
              class="control-btn primary"
              disabled={loadingModel}
              onclick={() => void loadSearchModel()}
            >
              {#if loadingModel}
                Loading model…
              {:else if searchStatus?.embedderLoaded}
                Re-embed vault
              {:else}
                Load MiniLM model
              {/if}
            </button>
            <button
              type="button"
              class="control-btn"
              disabled={loadingModel}
              onclick={() => void refreshSearchStatus()}
            >
              Refresh status
            </button>
          </div>
          <p class="hint">
            Without the model, Dense/Binary settings have no effect — only
            keyword search runs. Loading downloads ~90&nbsp;MB once, then
            re-embeds every prompt so ANN can rank them.
          </p>
        </section>

        <section class="panel" aria-labelledby="recency-heading">
          <div class="panel-head">
            <h3 id="recency-heading">Recency half-life</h3>
            <p class="help">
              Days until a prompt’s recency boost falls to ~37%. Default 30.
            </p>
          </div>
          <label class="field">
            <span class="field-label">Days</span>
            <input type="number" min="1" max="3650" bind:value={recencyDays} />
          </label>
          <div class="row-actions">
            <button type="button" class="control-btn primary" onclick={() => void saveRecency()}>
              Save recency
            </button>
          </div>
        </section>

        <section class="panel" aria-labelledby="embedding-heading">
          <div class="panel-head">
            <h3 id="embedding-heading">Embedding quantization</h3>
            <p class="help">
              Trade-off between recall and search speed on the
              <code>prompts.embedding</code> ANN index. Requires the model above.
            </p>
          </div>
          <fieldset class="radio-group" disabled={$rebuildingIndex || searchStatus?.sparseOnly}>
            <legend class="visually-hidden">Embedding quantization mode</legend>
            <label class="radio-card" class:selected={pendingQuant === 'binary'}>
              <input
                type="radio"
                name="embedding-quant"
                value="binary"
                bind:group={pendingQuant}
                onchange={() => pickQuant('binary')}
              />
              <span class="radio-copy">
                <span class="radio-label">Binary <span class="badge">Default</span></span>
                <span class="radio-desc">
                  Binary HNSW candidates + exact cosine rerank. Fast; low memory.
                </span>
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
              <span class="radio-copy">
                <span class="radio-label">Dense</span>
                <span class="radio-desc">
                  Full-precision Dense ANN after index rebuild; exact cosine while rebuild is pending.
                </span>
              </span>
            </label>
          </fieldset>
          {#if $rebuildingIndex}
            <p class="hint status" role="status">
              <span class="pulse" aria-hidden="true"></span>
              Updating embedding index…
            </p>
          {/if}
          {#if searchStatus?.sparseOnly}
            <p class="hint">Load the MiniLM model first — ANN modes stay inactive until then.</p>
          {/if}
        </section>
      {:else if active === 'vault'}
        <section class="panel" aria-labelledby="autolock-heading">
          <div class="panel-head">
            <h3 id="autolock-heading">Auto-lock</h3>
            <p class="help">Clear the vault session after idle time, on quit, or never.</p>
          </div>
          <label class="field">
            <span class="field-label">Policy</span>
            <select bind:value={autoLockPolicy}>
              <option value="lock_on_quit">Lock on quit only</option>
              <option value="idle">Idle timeout</option>
              <option value="disabled">Disabled</option>
            </select>
          </label>
          {#if autoLockPolicy === 'idle'}
            <label class="field">
              <span class="field-label">Idle minutes</span>
              <input type="number" min="1" max="1440" bind:value={autoLockIdleMinutes} />
            </label>
          {/if}
          <div class="row-actions">
            <button type="button" class="control-btn primary" onclick={() => void saveAutoLock()}>
              Save auto-lock
            </button>
          </div>
        </section>
        <section class="panel" aria-labelledby="history-heading">
          <div class="panel-head">
            <h3 id="history-heading">History retention</h3>
            <p class="help">Days of per-prompt snapshots kept under <code>.onq/history</code>.</p>
          </div>
          <label class="field">
            <span class="field-label">Days</span>
            <input type="number" min="0" max="3650" bind:value={historyDays} />
          </label>
          <div class="row-actions">
            <button
              type="button"
              class="control-btn primary"
              onclick={() => void saveHistoryRetention()}
            >
              Save retention
            </button>
          </div>
        </section>
        <section class="panel">
          <div class="panel-head">
            <h3>Import / export prompts</h3>
            <p class="help">Bulk import Markdown/JSON/ChatGPT export, or export the vault as .md files.</p>
          </div>
          <div class="row-actions">
            <button type="button" class="control-btn primary" onclick={() => void runImport()}>
              Import…
            </button>
            <button type="button" class="control-btn" onclick={() => void runExport()}>
              Export…
            </button>
          </div>
          {#if importStatus}
            <p class="hint" role="status">{importStatus}</p>
          {/if}
        </section>
        {#if authMode === 'keychain'}
          <section class="panel">
            <div class="panel-head">
              <h3>Encryption key</h3>
              <p class="help">
                This vault uses a key stored in your system keychain. Enter its
                recovery phrase only when you need to display that key.
              </p>
            </div>
            {#if encryptionKey}
              <textarea class="secure" readonly rows="3" value={encryptionKey}></textarea>
              <p class="hint">Keep this key private. Close settings when finished.</p>
            {:else}
              <textarea
                class="secure"
                rows="3"
                bind:value={recoveryPhrase}
                placeholder="24-word recovery phrase"
                autocomplete="off"
                spellcheck="false"
              ></textarea>
              <button
                type="button"
                class="control-btn primary"
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
            <div class="panel-head">
              <h3>Vault security</h3>
              <p class="help">
                Password-protected vault. Unlock when prompted; auto-lock policies
                apply based on your vault settings.
              </p>
            </div>
            <div class="status-pill">
              Auth mode: <strong>{authMode ?? 'unknown'}</strong>
            </div>
          </section>
        {/if}
      {:else if active === 'backups'}
        {#if backupRemind}
          <p class="hint" role="status">Vault backup may be overdue — export a fresh .onqbak when you can.</p>
        {/if}
        <BackupsSection {onVaultClosed} />
      {:else if active === 'plugins'}
        <PluginsSection />
      {:else if active === 'updates'}
        <section class="panel">
          <div class="panel-head">
            <h3>Beta channel</h3>
            <p class="help">
              Receive pre-release auto-updates earlier. Production
              <code>latest.json</code> remains the default feed until beta
              routing ships.
            </p>
          </div>
          <label class="toggle-row">
            <span class="toggle-copy">
              <span class="toggle-label">Opt in to beta releases</span>
              <span class="toggle-desc">Earlier builds, more frequent updates</span>
            </span>
            <span class="switch" class:on={pendingBeta}>
              <input
                type="checkbox"
                bind:checked={pendingBeta}
                onchange={(event) => flipBeta(event.currentTarget.checked)}
              />
              <span class="switch-track" aria-hidden="true">
                <span class="switch-thumb"></span>
              </span>
            </span>
          </label>
          <div class="row-actions">
            <button
              type="button"
              class="control-btn primary"
              disabled={checkingForUpdates}
              onclick={() => void runUpdateCheck()}
            >
              {checkingForUpdates ? 'Checking for updates…' : 'Check for Updates'}
            </button>
          </div>
          {#if updateStatus}
            <p class="hint status" role="status">{updateStatus}</p>
          {/if}
        </section>
      {/if}

      {#if errorMessage}
        <p class="error banner" role="alert">{errorMessage}</p>
      {/if}
    </div>
  </div>

</div>

<style>
  .settings-page {
    position: relative;
    box-sizing: border-box;
    width: 100%;
    margin: 0;
    padding: 28px 28px 72px;
    color: var(--glass-text);
    overflow: hidden;
  }
  .page-glow {
    position: absolute;
    top: -140px;
    left: -60px;
    width: 380px;
    height: 300px;
    border-radius: 50%;
    background: radial-gradient(circle, color-mix(in srgb, var(--glass-cyan) 22%, transparent), transparent 70%);
    pointer-events: none;
  }
  .settings-top {
    position: relative;
    margin-bottom: 24px;
  }
  .eyebrow {
    margin: 0 0 6px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--glass-cyan);
  }
  h1 {
    margin: 0 0 6px;
    font-size: 32px;
    font-weight: 700;
    letter-spacing: -0.03em;
    line-height: 1.1;
  }
  h2 {
    margin: 0 0 2px;
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  h3 {
    margin: 0 0 6px;
    font-size: 15px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .sub,
  .help,
  .section-sub {
    margin: 0;
    color: var(--glass-text-dim);
    font-size: 14px;
    line-height: 1.5;
  }
  .help {
    font-size: 13px;
    margin-bottom: 0;
  }
  .help code {
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 12px;
    padding: 1px 5px;
    border-radius: 5px;
    background: var(--glass-control-bg);
    border: 1px solid var(--glass-border);
  }
  .settings-layout {
    position: relative;
    display: grid;
    grid-template-columns: 220px minmax(0, 1fr);
    gap: 20px;
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
    padding: 10px;
    border-radius: 18px;
    border: 1px solid var(--glass-border);
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.03), transparent 30%),
      var(--glass-inset);
    box-shadow: var(--glass-shadow-md), var(--glass-inset-highlight);
    position: sticky;
    top: 16px;
  }
  .nav-item {
    appearance: none;
    border: 0;
    background: transparent;
    color: var(--glass-text-dim);
    text-align: left;
    padding: 12px 14px;
    border-radius: 12px;
    font: inherit;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 2px;
    transition:
      background var(--motion-duration) ease,
      color var(--motion-duration) ease,
      box-shadow var(--motion-duration) ease;
  }
  .nav-label {
    font-size: 13px;
    font-weight: 650;
  }
  .nav-hint {
    font-size: 11px;
    color: var(--glass-text-faint);
  }
  .nav-item.active {
    color: var(--glass-selected-fg);
    background: var(--glass-selected-bg);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--glass-selected-fg) 22%, transparent);
  }
  .nav-item.active .nav-hint {
    color: color-mix(in srgb, var(--glass-selected-fg) 75%, var(--glass-text-faint));
  }
  .nav-item:hover {
    background: var(--glass-hover);
    color: var(--glass-text);
  }
  .nav-item:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .settings-main {
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-width: 0;
  }
  .section-head {
    margin-bottom: 2px;
  }
  .panel {
    position: relative;
    padding: 20px;
    border-radius: 16px;
    border: 1px solid var(--glass-border);
    background:
      linear-gradient(165deg, rgba(255, 255, 255, 0.035), transparent 42%),
      var(--glass-panel);
    box-shadow: var(--glass-inset-highlight);
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .panel-head {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .control-btn {
    appearance: none;
    border: 1px solid var(--glass-border);
    background: var(--glass-control-bg);
    color: var(--glass-text);
    border-radius: 12px;
    padding: 11px 16px;
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    align-self: flex-start;
    transition:
      background var(--motion-duration) ease,
      border-color var(--motion-duration) ease,
      box-shadow var(--motion-duration) ease,
      transform var(--motion-duration) var(--motion-spring);
  }
  .control-btn.primary {
    border-color: color-mix(in srgb, var(--glass-periwinkle) 38%, var(--glass-border));
    background: color-mix(in srgb, var(--glass-accent) 22%, var(--glass-control-bg));
    color: var(--glass-text);
    box-shadow: none;
  }
  .control-btn.primary:hover:not(:disabled) {
    background: color-mix(in srgb, var(--glass-accent) 32%, var(--glass-control-bg));
    border-color: color-mix(in srgb, var(--glass-periwinkle) 55%, var(--glass-border));
  }
  .control-btn.shortcut-btn {
    min-width: 200px;
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 13px;
    letter-spacing: 0.02em;
  }
  .control-btn.recording {
    border-color: color-mix(in srgb, var(--glass-periwinkle) 55%, transparent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--glass-periwinkle) 16%, transparent);
    color: var(--glass-periwinkle);
  }
  .control-btn:hover:not(:disabled) {
    background: var(--glass-hover-strong);
  }
  .control-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .control-btn:focus-visible,
  .theme-card:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .shortcut-btn kbd {
    font: inherit;
  }
  .rec-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--glass-danger);
    box-shadow: 0 0 10px var(--glass-danger);
    animation: pulse 1.2s ease infinite;
  }
  @keyframes pulse {
    50% {
      opacity: 0.4;
    }
  }
  .hint {
    margin: 0;
    font-size: 12px;
    color: var(--glass-text-dim);
    line-height: 1.45;
  }
  .hint.status {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .pulse {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--glass-periwinkle);
    box-shadow: 0 0 10px var(--glass-periwinkle);
    animation: pulse 1.2s ease infinite;
  }
  .error {
    margin: 0;
    color: var(--glass-danger);
    font-size: 13px;
  }
  .error.banner {
    padding: 12px 14px;
    border-radius: 12px;
    border: 1px solid var(--glass-danger-border);
    background: var(--glass-danger-bg);
  }
  .theme-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .theme-card {
    appearance: none;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border-radius: 12px;
    border: 1px solid var(--glass-border);
    background: var(--glass-control-bg);
    color: var(--glass-text);
    cursor: pointer;
    font: inherit;
    text-align: left;
    transition:
      border-color var(--motion-duration) ease,
      background var(--motion-duration) ease,
      box-shadow var(--motion-duration) ease;
  }
  .theme-card.selected {
    border-color: color-mix(in srgb, var(--glass-selected-fg) 45%, transparent);
    background: var(--glass-selected-bg);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--glass-selected-fg) 18%, transparent);
  }
  .theme-card:hover {
    background: var(--glass-hover-strong);
  }
  .theme-swatch {
    width: 28px;
    height: 28px;
    border-radius: 9px;
    border: 1px solid var(--glass-border-strong);
    flex-shrink: 0;
  }
  .theme-swatch.dark {
    background: linear-gradient(135deg, #1a2138, #2a3454);
  }
  .theme-swatch.light {
    background: linear-gradient(135deg, #f0f4ff, #ffffff);
  }
  .theme-label {
    font-size: 13px;
    font-weight: 650;
  }
  .radio-group {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: 0;
    padding: 0;
    border: 0;
  }
  .radio-card {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    padding: 14px 14px;
    border: 1px solid var(--glass-border);
    border-radius: 12px;
    cursor: pointer;
    background: var(--glass-control-bg);
    transition:
      border-color var(--motion-duration) ease,
      background var(--motion-duration) ease,
      box-shadow var(--motion-duration) ease;
  }
  .radio-card.selected {
    border-color: color-mix(in srgb, var(--glass-selected-fg) 45%, transparent);
    background: var(--glass-selected-bg);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--glass-selected-fg) 18%, transparent);
  }
  .radio-card:hover {
    border-color: var(--glass-border-strong);
  }
  .radio-card input {
    margin-top: 3px;
    accent-color: var(--glass-accent);
  }
  .radio-copy {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }
  .radio-label {
    font-weight: 650;
    font-size: 14px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .badge {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    padding: 2px 7px;
    border-radius: 999px;
    color: var(--glass-selected-fg);
    background: color-mix(in srgb, var(--glass-selected-fg) 14%, transparent);
    border: 1px solid color-mix(in srgb, var(--glass-selected-fg) 28%, transparent);
  }
  .radio-desc {
    font-size: 12px;
    color: var(--glass-text-dim);
    line-height: 1.45;
  }
  .toggle-row {
    display: flex;
    gap: 14px;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    border: 1px solid var(--glass-border);
    border-radius: 12px;
    cursor: pointer;
    background: var(--glass-control-bg);
  }
  .toggle-copy {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .toggle-label {
    font-size: 13px;
    font-weight: 650;
    line-height: 1.4;
    color: var(--glass-text);
  }
  .toggle-desc {
    font-size: 12px;
    color: var(--glass-text-dim);
  }
  .switch {
    position: relative;
    flex-shrink: 0;
  }
  .switch input {
    position: absolute;
    inset: 0;
    opacity: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    cursor: pointer;
  }
  .switch-track {
    display: block;
    width: 44px;
    height: 26px;
    border-radius: 999px;
    background: var(--glass-hover-strong);
    border: 1px solid var(--glass-border-strong);
    transition: background var(--motion-duration) ease, border-color var(--motion-duration) ease;
    position: relative;
  }
  .switch-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.25);
    transition: transform var(--motion-duration) var(--motion-spring);
  }
  .switch.on .switch-track {
    background: var(--glass-accent);
    border-color: transparent;
  }
  .switch.on .switch-thumb {
    transform: translateX(18px);
  }
  .switch input:focus-visible + .switch-track {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .secure {
    width: 100%;
    box-sizing: border-box;
    border-radius: 12px;
    border: 1px solid var(--glass-border);
    background: var(--glass-input);
    color: var(--glass-text);
    padding: 12px 14px;
    font: inherit;
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 12px;
    line-height: 1.5;
    resize: vertical;
    min-height: 84px;
  }
  .secure:focus-visible {
    outline: none;
    border-color: color-mix(in srgb, var(--glass-periwinkle) 55%, transparent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--glass-periwinkle) 16%, transparent);
  }
  .status-pill {
    align-self: flex-start;
    padding: 8px 12px;
    border-radius: 999px;
    border: 1px solid var(--glass-border);
    background: var(--glass-control-bg);
    font-size: 12px;
    color: var(--glass-text-dim);
  }
  .status-pill strong {
    color: var(--glass-text);
    font-weight: 650;
  }
  .status-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .status-list li {
    display: grid;
    grid-template-columns: 140px minmax(0, 1fr);
    gap: 4px 12px;
    align-items: baseline;
    padding: 10px 12px;
    border-radius: 12px;
    border: 1px solid var(--glass-border);
    background: var(--glass-control-bg);
  }
  .status-k {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--glass-text-faint);
  }
  .status-v {
    font-size: 13px;
    font-weight: 650;
    color: var(--glass-text);
  }
  .status-v.ok {
    color: var(--glass-selected-fg);
  }
  .status-v.warn {
    color: var(--glass-gold);
  }
  .status-note {
    grid-column: 2;
    font-size: 12px;
    color: var(--glass-text-dim);
  }
  .status-note.mono {
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 11px;
  }
  .row-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
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
  @media (prefers-reduced-motion: reduce) {
    .rec-dot,
    .pulse {
      animation: none;
    }
  }
</style>
