<script lang="ts">
  /**
   * Settings → Backups panel: vault/DB paths, export, destructive import.
   * Emits `onVaultClosed` after a successful import so App can show unlock.
   */
  import { onMount } from 'svelte';
  import {
    backupIsSealed,
    exportVaultBackup,
    getBackupPaths,
    importVaultBackupFromPath,
    pickBackupArchive,
    type BackupPaths,
    type ImportBackupResult,
  } from '$lib/api/backup';
  import ConfirmDialog from '$lib/components/primitives/ConfirmDialog.svelte';
  import { t, locale } from '$lib/i18n';

  let {
    onVaultClosed,
  }: {
    /** Fired after import replaces the vault and closes the session. */
    onVaultClosed?: (importResult: ImportBackupResult) => void; // eslint-disable-line no-unused-vars
  } = $props();

  let paths = $state<BackupPaths | null>(null);
  let loadError = $state<string | null>(null);
  let actionError = $state<string | null>(null);
  let status = $state<string | null>(null);

  let exportPassword = $state('');
  let exportConfirm = $state('');
  let protectExport = $state(false);

  let importPassword = $state('');
  let busy = $state(false);

  let confirmImportOpen = $state(false);
  let pendingArchive = $state<string | null>(null);
  let pendingSealed = $state(false);

  onMount(() => {
    void refreshPaths();
  });

  async function refreshPaths() {
    loadError = null;
    try {
      paths = await getBackupPaths();
    } catch (e) {
      paths = null;
      loadError = e instanceof Error ? e.message : String(e);
    }
  }

  async function copyText(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      status = t('backup.pathCopied');
      setTimeout(() => {
        if (status === t('backup.pathCopied')) status = null;
      }, 2_000);
    } catch {
      status = t('backup.pathCopyFailed');
    }
  }

  async function runExport() {
    if (busy) return;
    actionError = null;
    status = null;
    if (protectExport) {
      if (!exportPassword.trim()) {
        actionError = t('backup.needPassword');
        return;
      }
      if (exportPassword !== exportConfirm) {
        actionError = t('backup.passwordMismatch');
        return;
      }
    }
    busy = true;
    try {
      const dest = await exportVaultBackup(protectExport ? exportPassword : null);
      if (dest) {
        status = t('backup.writtenTo', { path: dest });
        exportPassword = '';
        exportConfirm = '';
      }
    } catch (e) {
      actionError = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function startImport() {
    if (busy) return;
    actionError = null;
    status = null;
    try {
      const archive = await pickBackupArchive();
      if (!archive) return;
      pendingArchive = archive;
      pendingSealed = await backupIsSealed(archive);
      importPassword = '';
      confirmImportOpen = true;
    } catch (e) {
      actionError = e instanceof Error ? e.message : String(e);
    }
  }

  async function confirmImport() {
    if (!pendingArchive || busy) return;
    if (pendingSealed && !importPassword.trim()) {
      actionError = t('backup.sealedNeedPassword');
      return;
    }
    busy = true;
    actionError = null;
    try {
      const result = await importVaultBackupFromPath(
        pendingArchive,
        pendingSealed ? importPassword : null,
      );
      confirmImportOpen = false;
      pendingArchive = null;
      onVaultClosed?.(result);
    } catch (e) {
      actionError = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function cancelImport() {
    if (busy) return;
    confirmImportOpen = false;
    pendingArchive = null;
    importPassword = '';
  }
</script>

<section class="panel" aria-labelledby="backup-paths-heading">
  <div class="panel-head">
    <h3 id="backup-paths-heading">{t('backup.locations', undefined, $locale)}</h3>
    <p class="help">{t('backup.locationsHelp', undefined, $locale)}</p>
  </div>

  {#if loadError}
    <p class="error" role="alert">{loadError}</p>
  {:else if paths}
    {@const vaultPath = paths.vaultPath}
    {@const databasePath = paths.databasePath}
    <div class="toggle-row field-row path-field-row">
      <span class="toggle-copy">
        <span class="toggle-label">{t('backup.vault', undefined, $locale)}</span>
      </span>
      <div class="field-row-actions path-actions">
        <code class="path mono" title={vaultPath}>{vaultPath}</code>
        <button type="button" class="control-btn slim" onclick={() => void copyText(vaultPath)}>
          {t('editor.copyAction', undefined, $locale)}
        </button>
      </div>
    </div>
    <div class="toggle-row field-row path-field-row">
      <span class="toggle-copy">
        <span class="toggle-label">{t('backup.database', undefined, $locale)}</span>
      </span>
      <div class="field-row-actions path-actions">
        <code class="path mono" title={databasePath}>{databasePath}</code>
        <button
          type="button"
          class="control-btn slim"
          onclick={() => void copyText(databasePath)}
        >
          {t('editor.copyAction', undefined, $locale)}
        </button>
      </div>
    </div>
  {:else}
    <p class="hint">{t('backup.loadingPaths', undefined, $locale)}</p>
  {/if}
</section>

<section class="panel" aria-labelledby="backup-export-heading">
  <div class="panel-head">
    <h3 id="backup-export-heading">{t('backup.exportHeading', undefined, $locale)}</h3>
    <p class="help">{t('backup.exportHelp', undefined, $locale)}</p>
  </div>

  <label class="toggle-row">
    <span class="toggle-copy">
      <span class="toggle-label">{t('backup.protect', undefined, $locale)}</span>
      <span class="toggle-desc">{t('backup.protectDesc', undefined, $locale)}</span>
    </span>
    <span class="switch" class:on={protectExport}>
      <input type="checkbox" bind:checked={protectExport} />
      <span class="switch-track" aria-hidden="true">
        <span class="switch-thumb"></span>
      </span>
    </span>
  </label>

  {#if protectExport}
    <label class="toggle-row field-row">
      <span class="toggle-copy">
        <span class="toggle-label"
          >{t('backup.archivePassword', undefined, $locale)}</span
        >
      </span>
      <input
        class="text-input"
        type="password"
        autocomplete="new-password"
        bind:value={exportPassword}
        disabled={busy}
        aria-label={t('backup.archivePassword', undefined, $locale)}
      />
    </label>
    <label class="toggle-row field-row">
      <span class="toggle-copy">
        <span class="toggle-label"
          >{t('backup.confirmPassword', undefined, $locale)}</span
        >
      </span>
      <input
        class="text-input"
        type="password"
        autocomplete="new-password"
        bind:value={exportConfirm}
        disabled={busy}
        aria-label={t('backup.confirmPassword', undefined, $locale)}
      />
    </label>
  {/if}

  <div class="toggle-row field-row">
    <span class="toggle-copy">
      <span class="toggle-label">{t('backup.exportHeading', undefined, $locale)}</span>
    </span>
    <button
      type="button"
      class="control-btn primary"
      disabled={busy || !paths}
      onclick={() => void runExport()}
    >
      {busy
        ? t('common.working', undefined, $locale)
        : t('backup.exportBtn', undefined, $locale)}
    </button>
  </div>
</section>

<section class="panel" aria-labelledby="backup-import-heading">
  <div class="panel-head">
    <h3 id="backup-import-heading">{t('backup.importHeading', undefined, $locale)}</h3>
    <p class="help">{t('backup.importHelp', undefined, $locale)}</p>
  </div>
  <div class="toggle-row field-row">
    <span class="toggle-copy">
      <span class="toggle-label">{t('backup.importHeading', undefined, $locale)}</span>
    </span>
    <button
      type="button"
      class="control-btn danger"
      disabled={busy || !paths}
      onclick={() => void startImport()}
    >
      {t('backup.importBtn', undefined, $locale)}
    </button>
  </div>
</section>

{#if status}
  <p class="hint status" role="status">{status}</p>
{/if}
{#if actionError}
  <p class="error" role="alert">{actionError}</p>
{/if}

<ConfirmDialog
  bind:open={confirmImportOpen}
  title={t('backup.replaceTitle', undefined, $locale)}
  description={t('backup.replaceDesc', undefined, $locale)}
  itemLabel={pendingArchive ?? ''}
  itemKind={t('backup.itemKind', undefined, $locale)}
  confirmLabel={t('backup.replaceConfirm', undefined, $locale)}
  busyLabel={t('backup.importing', undefined, $locale)}
  cancelLabel={t('common.cancel', undefined, $locale)}
  {busy}
  onConfirm={() => void confirmImport()}
  onCancel={cancelImport}
>
  {#if pendingSealed}
    <label class="field import-pw">
      <span class="field-label">{t('backup.archivePassword', undefined, $locale)}</span>
      <input
        class="text-input"
        type="password"
        autocomplete="current-password"
        bind:value={importPassword}
        disabled={busy}
      />
    </label>
  {/if}
</ConfirmDialog>

<style>
  /* Panel/button/toggle chrome comes from the shared settings-chrome.css
   * (scoped under .settings-page); only backup-specific styles live here. */
  .path-field-row {
    align-items: flex-start;
  }
  .path-actions {
    flex: 1 1 12rem;
    min-width: 0;
    max-width: min(420px, 58vw);
    justify-content: flex-end;
  }
  .path {
    flex: 1 1 auto;
    min-width: 0;
    display: block;
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid var(--glass-border);
    background: var(--glass-inset);
    font-size: 12px;
    line-height: 1.4;
    word-break: break-all;
    color: var(--glass-text);
  }
  .import-pw {
    margin-top: 12px;
  }
  .mono {
    font-family: 'JetBrains Mono', ui-monospace, monospace;
  }
</style>
