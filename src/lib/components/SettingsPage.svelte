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
  import AuditSection from '$lib/components/settings/AuditSection.svelte';
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
  import {
    getEmbedderPreference,
    listPlugins,
    setEmbedderPreference,
    type PluginInfo,
  } from '$lib/api/plugins';
  import {
    locale,
    setLocale,
    t,
    SUPPORTED_LOCALES,
    type Locale,
    type MessageKey,
  } from '$lib/i18n';

  type SectionId = 'general' | 'search' | 'vault' | 'backups' | 'plugins' | 'updates';

  const metaKeyLabel = metaModifierLabel();
  const sections = $derived([
    { id: 'general' as const, label: t('settings.general', undefined, $locale) },
    { id: 'search' as const, label: t('settings.search', undefined, $locale) },
    { id: 'vault' as const, label: t('settings.vault', undefined, $locale) },
    { id: 'backups' as const, label: t('settings.backups', undefined, $locale) },
    { id: 'plugins' as const, label: t('settings.plugins', undefined, $locale) },
    { id: 'updates' as const, label: t('settings.updates', undefined, $locale) },
  ]);
  let pendingLocale = $state<Locale>($locale);

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
  let embedderPref = $state('builtin');
  let embedderPlugins = $state<PluginInfo[]>([]);

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
    void getEmbedderPreference()
      .then((v) => {
        if (v) embedderPref = v;
      })
      .catch(() => undefined);
    void listPlugins()
      .then((ps) => {
        embedderPlugins = ps.filter((p) => {
          const caps = Array.isArray(p.capabilities)
            ? p.capabilities
            : typeof p.capabilities === 'string'
              ? [p.capabilities]
              : [];
          return (
            p.enabled &&
            caps.some(
              (c) =>
                typeof c === 'string' &&
                (c === 'embedding' || c === 'embedder' || c.startsWith('embedding')),
            )
          );
        });
      })
      .catch(() => undefined);
  });

  async function saveEmbedderPref() {
    await setEmbedderPreference(embedderPref);
  }

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
</script>

{#snippet navIcon(id: SectionId)}
  <svg class="nav-icon" viewBox="0 0 20 20" width="16" height="16" aria-hidden="true">
    {#if id === 'general'}
      <path d="M3 6h14M3 10h14M3 14h14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      <circle cx="13" cy="6" r="1.8" fill="currentColor" />
      <circle cx="7.5" cy="10" r="1.8" fill="currentColor" />
      <circle cx="11.5" cy="14" r="1.8" fill="currentColor" />
    {:else if id === 'search'}
      <circle cx="9" cy="9" r="5" fill="none" stroke="currentColor" stroke-width="1.5" />
      <path d="M12.8 12.8 16.5 16.5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
    {:else if id === 'vault'}
      <path d="M10 2.8 16 5.3v4.4c0 3.9-2.6 6.3-6 7.5-3.4-1.2-6-3.6-6-7.5V5.3L10 2.8Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
      <circle cx="10" cy="9.2" r="1.4" fill="none" stroke="currentColor" stroke-width="1.3" />
      <path d="M10 10.6v2.2" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
    {:else if id === 'backups'}
      <rect x="3.5" y="4" width="13" height="3.6" rx="1" fill="none" stroke="currentColor" stroke-width="1.5" />
      <path d="M5 7.6v7.2a1.2 1.2 0 0 0 1.2 1.2h7.6a1.2 1.2 0 0 0 1.2-1.2V7.6" fill="none" stroke="currentColor" stroke-width="1.5" />
      <path d="M8.3 10.8h3.4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
    {:else if id === 'plugins'}
      <rect x="4" y="4" width="5" height="5" rx="1.2" fill="none" stroke="currentColor" stroke-width="1.5" />
      <rect x="11" y="4" width="5" height="5" rx="1.2" fill="none" stroke="currentColor" stroke-width="1.5" />
      <rect x="4" y="11" width="5" height="5" rx="1.2" fill="none" stroke="currentColor" stroke-width="1.5" />
      <path d="M13.5 11.2v4.6M11.2 13.5h4.6" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
    {:else}
      <path d="M16.2 10a6.2 6.2 0 1 1-1.9-4.4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      <path d="M16.4 2.6v3.2h-3.2" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
    {/if}
  </svg>
{/snippet}

<div class="settings-page">
  <header class="settings-top">
    <p class="eyebrow">{t('settings.preferences', undefined, $locale)}</p>
    <h1>{t('settings.title', undefined, $locale)}</h1>
    <p class="sub">{t('settings.subtitle', undefined, $locale)}</p>
  </header>

  <div class="settings-layout">
    <nav class="settings-nav" aria-label={t('settings.nav', undefined, $locale)}>
      {#each sections as s (s.id)}
        <button
          type="button"
          class="nav-item"
          class:active={active === s.id}
          onclick={() => (active = s.id)}
        >
          {@render navIcon(s.id)}
          <span class="nav-label">{s.label}</span>
        </button>
      {/each}
    </nav>

    <div class="settings-main">
      {#if active === 'general'}
        <section class="panel" aria-labelledby="palette-heading">
          <div class="panel-head">
            <h3 id="palette-heading">{t('settings.palette', undefined, $locale)}</h3>
            <p class="help">{t('settings.paletteHelp', undefined, $locale)}</p>
          </div>
          <div class="toggle-row shortcut-row">
            <span class="toggle-copy">
              <span class="toggle-label" id="shortcut-heading"
                >{t('settings.shortcut', undefined, $locale)}</span
              >
              <span class="toggle-desc"
                >{t('settings.shortcutDesc', undefined, $locale)}</span
              >
            </span>
            <button
              type="button"
              class:recording={recordingShortcut}
              class="control-btn shortcut-btn"
              aria-labelledby="shortcut-heading"
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
          </div>
          {#if recordingShortcut}
            <p class="hint"
              >{t('settings.shortcutHint', { meta: metaKeyLabel }, $locale)}</p
            >
          {/if}
          {#if shortcutError}
            <p class="error" role="alert">{shortcutError}</p>
          {/if}
          <label class="toggle-row">
            <span class="toggle-copy">
              <span class="toggle-label"
                >{t('settings.minimizeOnCopy', undefined, $locale)}</span
              >
              <span class="toggle-desc"
                >{t('settings.minimizeOnCopyDesc', undefined, $locale)}</span
              >
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

        <section class="panel">
          <div class="panel-head">
            <h3>{t('settings.theme', undefined, $locale)}</h3>
            <p class="help">{t('settings.themeHelp', undefined, $locale)}</p>
          </div>
          <div class="theme-row">
            <button
              type="button"
              class="theme-card"
              class:selected={$theme === 'dark'}
              onclick={() => $theme !== 'dark' && toggleTheme()}
            >
              <span class="theme-swatch dark" aria-hidden="true"></span>
              <span class="theme-label">{t('settings.themeDark', undefined, $locale)}</span>
            </button>
            <button
              type="button"
              class="theme-card"
              class:selected={$theme === 'light'}
              onclick={() => $theme !== 'light' && toggleTheme()}
            >
              <span class="theme-swatch light" aria-hidden="true"></span>
              <span class="theme-label">{t('settings.themeLight', undefined, $locale)}</span>
            </button>
          </div>
        </section>

        <section class="panel" aria-labelledby="language-heading">
          <div class="panel-head">
            <h3 id="language-heading">{t('settings.language', undefined, $locale)}</h3>
            <p class="help">{t('settings.languageHelp', undefined, $locale)}</p>
          </div>
          <label class="toggle-row field-row">
            <span class="toggle-copy">
              <span class="toggle-label"
                >{t('settings.language', undefined, $locale)}</span
              >
            </span>
            <select
              bind:value={pendingLocale}
              aria-label={t('settings.language', undefined, $locale)}
              onchange={() => setLocale(pendingLocale)}
            >
              {#each SUPPORTED_LOCALES as loc (loc)}
                <option value={loc}
                  >{t(`locale.${loc}` as MessageKey, undefined, $locale)}</option
                >
              {/each}
            </select>
          </label>
        </section>

      {:else if active === 'search'}
        <section class="panel" aria-labelledby="embedding-heading">
          <div class="panel-head">
            <h3 id="embedding-heading"
              >{t('settings.quantHeading', undefined, $locale)}</h3
            >
            <p class="help">{t('settings.quantHelp', undefined, $locale)}</p>
          </div>
          <fieldset class="radio-group" disabled={$rebuildingIndex || searchStatus?.sparseOnly}>
            <legend class="visually-hidden"
              >{t('settings.quantLegend', undefined, $locale)}</legend
            >
            <label class="radio-card" class:selected={pendingQuant === 'binary'}>
              <input
                type="radio"
                name="embedding-quant"
                value="binary"
                bind:group={pendingQuant}
                onchange={() => pickQuant('binary')}
              />
              <span class="radio-copy">
                <span class="radio-label"
                  >{t('settings.binary', undefined, $locale)}
                  <span class="badge"
                    >{t('settings.defaultBadge', undefined, $locale)}</span
                  ></span
                >
                <span class="radio-desc"
                  >{t('settings.binaryDesc', undefined, $locale)}</span
                >
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
                <span class="radio-label">{t('settings.dense', undefined, $locale)}</span>
                <span class="radio-desc"
                  >{t('settings.denseDesc', undefined, $locale)}</span
                >
              </span>
            </label>
          </fieldset>
          {#if $rebuildingIndex}
            <p class="hint status" role="status">
              <span class="pulse" aria-hidden="true"></span>
              {t('settings.rebuildingIndex', undefined, $locale)}
            </p>
          {/if}
          {#if searchStatus?.sparseOnly}
            <p class="hint">{t('settings.loadMinilmFirst', undefined, $locale)}</p>
          {/if}
        </section>

        <section class="panel" aria-labelledby="search-status-heading">
          <div class="panel-head">
            <h3 id="search-status-heading"
              >{t('settings.searchHow', undefined, $locale)}</h3
            >
            <p class="help">{t('settings.searchHowHelp', undefined, $locale)}</p>
          </div>
          <ul class="status-list">
            <li>
              <span class="status-k">{t('settings.keywordIndex', undefined, $locale)}</span>
              <span class="status-v ok">{t('settings.statusOn', undefined, $locale)}</span>
              <span class="status-note"
                >{t('settings.keywordIndexNote', undefined, $locale)}</span
              >
            </li>
            <li>
              <span class="status-k"
                >{t('settings.embeddingModel', undefined, $locale)}</span
              >
              <span class="status-v" class:ok={searchStatus?.embedderLoaded} class:warn={!searchStatus?.embedderLoaded}>
                {#if searchStatus?.embedderLoaded}
                  {t('settings.statusLoaded', undefined, $locale)}
                {:else if searchStatus?.modelCached}
                  {t('settings.statusOnDisk', undefined, $locale)}
                {:else}
                  {t('settings.statusNotInstalled', undefined, $locale)}
                {/if}
              </span>
              <span class="status-note mono">{searchStatus?.modelId ?? 'sentence-transformers/all-MiniLM-L6-v2'}</span>
            </li>
            <li>
              <span class="status-k"
                >{t('settings.semanticPath', undefined, $locale)}</span
              >
              <span class="status-v" class:ok={!searchStatus?.sparseOnly} class:warn={searchStatus?.sparseOnly}>
                {#if searchStatus?.sparseOnly}
                  {t('settings.sparseOnly', undefined, $locale)}
                {:else if searchStatus?.embeddingQuant === 'dense' && searchStatus.denseReadiness === 'ready'}
                  {t('settings.denseLive', undefined, $locale)}
                {:else if searchStatus?.embeddingQuant === 'dense' && searchStatus.denseReadiness === 'pending'}
                  {t('settings.densePending', undefined, $locale)}
                {:else}
                  {t('settings.binaryAnn', undefined, $locale)}
                {/if}
              </span>
            </li>
          </ul>
          {#if searchStatusError}
            <p class="error" role="alert">{searchStatusError}</p>
          {/if}
          <div class="toggle-row field-row">
            <span class="toggle-copy">
              <span class="toggle-label"
                >{t('settings.embeddingModel', undefined, $locale)}</span
              >
              <span class="toggle-desc"
                >{t('settings.searchModelHint', undefined, $locale)}</span
              >
            </span>
            <div class="field-row-actions">
              <button
                type="button"
                class="control-btn primary"
                disabled={loadingModel}
                onclick={() => void loadSearchModel()}
              >
                {#if loadingModel}
                  {t('settings.loadingModel', undefined, $locale)}
                {:else if searchStatus?.embedderLoaded}
                  {t('settings.reembed', undefined, $locale)}
                {:else}
                  {t('settings.loadMinilm', undefined, $locale)}
                {/if}
              </button>
              <button
                type="button"
                class="control-btn"
                disabled={loadingModel}
                onclick={() => void refreshSearchStatus()}
              >
                {t('common.refresh', undefined, $locale)}
              </button>
            </div>
          </div>
        </section>

        <section class="panel" aria-labelledby="embedder-heading">
          <div class="panel-head">
            <h3 id="embedder-heading"
              >{t('settings.embedderHeading', undefined, $locale)}</h3
            >
            <p class="help">{t('settings.embedderHelp', undefined, $locale)}</p>
          </div>
          <label class="toggle-row field-row">
            <span class="toggle-copy">
              <span class="toggle-label"
                >{t('settings.activeEmbedder', undefined, $locale)}</span
              >
            </span>
            <select
              bind:value={embedderPref}
              aria-label={t('settings.activeEmbedder', undefined, $locale)}
              onchange={() => void saveEmbedderPref()}
            >
              <option value="builtin">builtin (all-MiniLM-L6-v2)</option>
              {#each embedderPlugins as p (p.id)}
                <option value={p.id}>{p.name} ({p.id})</option>
              {/each}
            </select>
          </label>
        </section>

        <section class="panel" aria-labelledby="recency-heading">
          <div class="panel-head">
            <h3 id="recency-heading">{t('settings.recency', undefined, $locale)}</h3>
            <p class="help">{t('settings.recencyHelp', undefined, $locale)}</p>
          </div>
          <label class="toggle-row field-row">
            <span class="toggle-copy">
              <span class="toggle-label"
                >{t('settings.recencyDays', undefined, $locale)}</span
              >
            </span>
            <input
              type="number"
              min="1"
              max="3650"
              bind:value={recencyDays}
              aria-label={t('settings.recencyDays', undefined, $locale)}
              onchange={() => void saveRecency()}
            />
          </label>
        </section>
      {:else if active === 'vault'}
        <section class="panel" aria-labelledby="autolock-heading">
          <div class="panel-head">
            <h3 id="autolock-heading">{t('settings.autoLock', undefined, $locale)}</h3>
            <p class="help">{t('settings.autoLockHelp', undefined, $locale)}</p>
          </div>
          <label class="toggle-row field-row">
            <span class="toggle-copy">
              <span class="toggle-label"
                >{t('settings.autoLockPolicy', undefined, $locale)}</span
              >
            </span>
            <select
              bind:value={autoLockPolicy}
              aria-label={t('settings.autoLockPolicy', undefined, $locale)}
              onchange={() => void saveAutoLock()}
            >
              <option value="lock_on_quit">{t('settings.autoLockQuit', undefined, $locale)}</option>
              <option value="idle">{t('settings.autoLockIdle', undefined, $locale)}</option>
              <option value="disabled">{t('settings.autoLockDisabled', undefined, $locale)}</option>
            </select>
          </label>
          {#if autoLockPolicy === 'idle'}
            <label class="toggle-row field-row">
              <span class="toggle-copy">
                <span class="toggle-label"
                  >{t('settings.autoLockMinutes', undefined, $locale)}</span
                >
              </span>
              <input
                type="number"
                min="1"
                max="1440"
                bind:value={autoLockIdleMinutes}
                aria-label={t('settings.autoLockMinutes', undefined, $locale)}
                onchange={() => void saveAutoLock()}
              />
            </label>
          {/if}
        </section>
        <section class="panel" aria-labelledby="history-heading">
          <div class="panel-head">
            <h3 id="history-heading">{t('settings.historyRetention', undefined, $locale)}</h3>
            <p class="help">{t('settings.historyHelp', undefined, $locale)}</p>
          </div>
          <label class="toggle-row field-row">
            <span class="toggle-copy">
              <span class="toggle-label"
                >{t('settings.recencyDays', undefined, $locale)}</span
              >
            </span>
            <input
              type="number"
              min="0"
              max="3650"
              bind:value={historyDays}
              aria-label={t('settings.recencyDays', undefined, $locale)}
              onchange={() => void saveHistoryRetention()}
            />
          </label>
        </section>
        <section class="panel">
          <div class="panel-head">
            <h3>{t('settings.importExport', undefined, $locale)}</h3>
            <p class="help">{t('settings.importExportHelp', undefined, $locale)}</p>
          </div>
          <div class="toggle-row field-row">
            <span class="toggle-copy">
              <span class="toggle-label"
                >{t('settings.importExport', undefined, $locale)}</span
              >
            </span>
            <div class="field-row-actions">
              <button type="button" class="control-btn primary" onclick={() => void runImport()}>
                {t('settings.import', undefined, $locale)}
              </button>
              <button type="button" class="control-btn" onclick={() => void runExport()}>
                {t('settings.export', undefined, $locale)}
              </button>
            </div>
          </div>
          {#if importStatus}
            <p class="hint" role="status">{importStatus}</p>
          {/if}
        </section>
        <AuditSection />
        {#if authMode === 'keychain'}
          <section class="panel">
            <div class="panel-head">
              <h3>{t('settings.encryptionKey', undefined, $locale)}</h3>
              <p class="help"
                >{t('settings.encryptionKeyHelp', undefined, $locale)}</p
              >
            </div>
            {#if encryptionKey}
              <textarea class="secure" readonly rows="3" value={encryptionKey}></textarea>
              <p class="hint">{t('settings.keepKeyPrivate', undefined, $locale)}</p>
            {:else}
              <textarea
                class="secure"
                rows="3"
                bind:value={recoveryPhrase}
                placeholder={t('settings.recoveryPlaceholder', undefined, $locale)}
                autocomplete="off"
                spellcheck="false"
              ></textarea>
              <div class="toggle-row field-row">
                <span class="toggle-copy">
                  <span class="toggle-label"
                    >{t('settings.encryptionKey', undefined, $locale)}</span
                  >
                </span>
                <button
                  type="button"
                  class="control-btn primary"
                  disabled={retrievingKey || !recoveryPhrase.trim()}
                  onclick={() => void showEncryptionKey()}
                >
                  {retrievingKey
                    ? t('settings.checkingKey', undefined, $locale)
                    : t('settings.showKey', undefined, $locale)}
                </button>
              </div>
            {/if}
            {#if keyError}<p class="error" role="alert">{keyError}</p>{/if}
          </section>
        {:else}
          <section class="panel">
            <div class="panel-head">
              <h3>{t('settings.vaultSecurity', undefined, $locale)}</h3>
              <p class="help"
                >{t('settings.vaultSecurityHelp', undefined, $locale)}</p
              >
            </div>
            <div class="status-pill">
              {t('settings.authMode', { mode: authMode ?? 'unknown' }, $locale)}
            </div>
          </section>
        {/if}
      {:else if active === 'backups'}
        {#if backupRemind}
          <p class="hint" role="status"
            >{t('settings.backupOverdue', undefined, $locale)}</p
          >
        {/if}
        <BackupsSection {onVaultClosed} />
      {:else if active === 'plugins'}
        <PluginsSection />
      {:else if active === 'updates'}
        <section class="panel">
          <div class="panel-head">
            <h3>{t('settings.betaChannel', undefined, $locale)}</h3>
            <p class="help">{t('settings.betaHelp', undefined, $locale)}</p>
          </div>
          <label class="toggle-row">
            <span class="toggle-copy">
              <span class="toggle-label"
                >{t('settings.betaOptIn', undefined, $locale)}</span
              >
              <span class="toggle-desc"
                >{t('settings.betaDesc', undefined, $locale)}</span
              >
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
          <div class="toggle-row field-row">
            <span class="toggle-copy">
              <span class="toggle-label"
                >{t('settings.updates', undefined, $locale)}</span
              >
            </span>
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
  }
  .settings-top {
    position: relative;
    margin-bottom: 28px;
  }
  .eyebrow {
    margin: 0 0 6px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--glass-cyan);
  }
  h1 {
    margin: 0 0 6px;
    font-size: 34px;
    font-weight: 700;
    letter-spacing: -0.03em;
    line-height: 1.1;
    background: linear-gradient(100deg, var(--glass-text) 35%, var(--glass-selected-fg));
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }
  :global(:root.light) h1 {
    background: linear-gradient(100deg, #0a1430 40%, #0e7490);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }
  .sub {
    margin: 0;
    color: var(--glass-text-dim);
    font-size: 14px;
  }
  .settings-layout {
    position: relative;
    display: grid;
    grid-template-columns: 216px minmax(0, 1fr);
    gap: 22px;
    align-items: start;
  }
  @media (max-width: 800px) {
    .settings-layout {
      grid-template-columns: 1fr;
    }
  }

  /* ---- Nav rail ---- */
  .settings-nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px;
    border-radius: var(--glass-radius-lg);
    border: 1px solid var(--glass-border);
    background:
      linear-gradient(180deg, rgba(160, 190, 255, 0.05), transparent 30%),
      var(--glass-panel);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    box-shadow: var(--glass-shadow-md), var(--glass-inset-highlight);
    position: sticky;
    top: 16px;
  }
  .nav-item {
    position: relative;
    appearance: none;
    border: 1px solid transparent;
    background: transparent;
    color: var(--glass-text-dim);
    text-align: left;
    padding: 10px 12px;
    border-radius: 12px;
    font: inherit;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 10px;
    transition:
      background var(--motion-duration) ease,
      color var(--motion-duration) ease,
      border-color var(--motion-duration) ease,
      box-shadow var(--motion-duration) ease;
  }
  .nav-icon {
    display: block;
    flex-shrink: 0;
    opacity: 0.75;
    transition: opacity var(--motion-duration) ease;
  }
  .nav-label {
    font-size: 13px;
    font-weight: 650;
  }
  .nav-item:hover {
    background: var(--glass-hover);
    color: var(--glass-text);
  }
  .nav-item:hover .nav-icon {
    opacity: 1;
  }
  .nav-item.active {
    color: var(--glass-selected-fg);
    border-color: color-mix(in srgb, var(--glass-accent-2) 28%, transparent);
    background: linear-gradient(
      90deg,
      color-mix(in srgb, var(--glass-accent-2) 14%, transparent),
      color-mix(in srgb, var(--glass-accent-2) 4%, transparent)
    );
    box-shadow: 0 0 18px color-mix(in srgb, var(--glass-accent-2) 12%, transparent);
  }
  .nav-item.active .nav-icon {
    opacity: 1;
  }
  /* Accent bar along the active item's leading edge. */
  .nav-item.active::before {
    content: '';
    position: absolute;
    left: 0;
    top: 9px;
    bottom: 9px;
    width: 3px;
    border-radius: 999px;
    background: var(--glass-gradient-accent);
    box-shadow: 0 0 8px var(--glass-glow-cyan);
  }
  .nav-item:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  @media (max-width: 800px) {
    .settings-nav {
      flex-direction: row;
      overflow-x: auto;
      position: static;
    }
    .nav-item {
      flex: 0 0 auto;
    }
    .nav-item.active::before {
      left: 12px;
      right: 12px;
      top: auto;
      bottom: 3px;
      width: auto;
      height: 2px;
    }
  }

  .settings-main {
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-width: 0;
  }

  /* ---- Shortcut recorder (nested in Palette panel) ---- */
  .shortcut-row {
    cursor: default;
  }
  .shortcut-row .control-btn.shortcut-btn {
    align-self: center;
    flex-shrink: 0;
    min-width: 120px;
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 13px;
    letter-spacing: 0.02em;
  }
  .control-btn.recording {
    border-color: color-mix(in srgb, var(--glass-accent-2) 55%, transparent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--glass-accent-2) 16%, transparent);
    color: var(--glass-selected-fg);
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
  .hint.status {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .pulse {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--glass-accent-2);
    box-shadow: 0 0 10px var(--glass-accent-2);
    animation: pulse 1.2s ease infinite;
  }

  /* ---- Theme picker ---- */
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
    border-color: color-mix(in srgb, var(--glass-accent-2) 45%, transparent);
    background: var(--glass-selected-bg);
    box-shadow:
      inset 0 0 0 1px color-mix(in srgb, var(--glass-accent-2) 18%, transparent),
      0 0 18px color-mix(in srgb, var(--glass-accent-2) 14%, transparent);
  }
  .theme-card:hover {
    background: var(--glass-hover-strong);
  }
  .theme-card:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .theme-swatch {
    width: 28px;
    height: 28px;
    border-radius: 9px;
    border: 1px solid var(--glass-border-strong);
    flex-shrink: 0;
  }
  .theme-swatch.dark {
    background: linear-gradient(135deg, #101a30, #2c3d63);
  }
  .theme-swatch.light {
    background: linear-gradient(135deg, #eef3fc, #ffffff);
  }
  .theme-label {
    font-size: 13px;
    font-weight: 650;
  }

  /* ---- Radio cards ---- */
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
    border-color: color-mix(in srgb, var(--glass-accent-2) 45%, transparent);
    background: var(--glass-selected-bg);
    box-shadow:
      inset 0 0 0 1px color-mix(in srgb, var(--glass-accent-2) 18%, transparent),
      0 0 18px color-mix(in srgb, var(--glass-accent-2) 14%, transparent);
  }
  .radio-card:hover {
    border-color: var(--glass-border-strong);
  }
  .radio-card input {
    margin-top: 3px;
    accent-color: var(--glass-accent-2);
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
    background: color-mix(in srgb, var(--glass-accent-2) 14%, transparent);
    border: 1px solid color-mix(in srgb, var(--glass-accent-2) 28%, transparent);
  }
  .radio-desc {
    font-size: 12px;
    color: var(--glass-text-dim);
    line-height: 1.45;
  }

  /* ---- Secure textarea ---- */
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
    border-color: color-mix(in srgb, var(--glass-accent-2) 55%, transparent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--glass-accent-2) 16%, transparent);
  }

  /* ---- Search status list ---- */
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
  /* Leading status dot, colored by state via currentColor. */
  .status-v.ok,
  .status-v.warn {
    display: inline-flex;
    align-items: center;
    gap: 7px;
  }
  .status-v.ok::before,
  .status-v.warn::before {
    content: '';
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: currentColor;
    box-shadow: 0 0 8px currentColor;
    flex-shrink: 0;
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
