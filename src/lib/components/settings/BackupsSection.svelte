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
      status = 'Path copied';
      setTimeout(() => {
        if (status === 'Path copied') status = null;
      }, 2_000);
    } catch {
      status = 'Could not copy path';
    }
  }

  async function runExport() {
    if (busy) return;
    actionError = null;
    status = null;
    if (protectExport) {
      if (!exportPassword.trim()) {
        actionError = 'Enter an archive password, or turn off password protection.';
        return;
      }
      if (exportPassword !== exportConfirm) {
        actionError = 'Archive password confirmation does not match.';
        return;
      }
    }
    busy = true;
    try {
      const dest = await exportVaultBackup(protectExport ? exportPassword : null);
      if (dest) {
        status = `Backup written to ${dest}`;
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
      actionError = 'This backup is password-protected. Enter the archive password.';
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
    <h3 id="backup-paths-heading">Vault locations</h3>
    <p class="help">
      Full vault root and the encrypted MongrelDB search index. Paths are absolute
      so you can find them in a file manager.
    </p>
  </div>

  {#if loadError}
    <p class="error" role="alert">{loadError}</p>
  {:else if paths}
    {@const vaultPath = paths.vaultPath}
    {@const databasePath = paths.databasePath}
    <dl class="path-list">
      <div class="path-row">
        <dt>Vault</dt>
        <dd>
          <code class="path mono" title={vaultPath}>{vaultPath}</code>
          <button type="button" class="control-btn slim" onclick={() => void copyText(vaultPath)}>
            Copy
          </button>
        </dd>
      </div>
      <div class="path-row">
        <dt>Database</dt>
        <dd>
          <code class="path mono" title={databasePath}>{databasePath}</code>
          <button
            type="button"
            class="control-btn slim"
            onclick={() => void copyText(databasePath)}
          >
            Copy
          </button>
        </dd>
      </div>
    </dl>
  {:else}
    <p class="hint">Loading paths…</p>
  {/if}
</section>

<section class="panel" aria-labelledby="backup-export-heading">
  <div class="panel-head">
    <h3 id="backup-export-heading">Export backup</h3>
    <p class="help">
      Creates a single <code>.onqbak</code> archive of the entire vault (prompts,
      history, encrypted index). Optional archive password is separate from your
      vault password.
    </p>
  </div>

  <label class="toggle-row">
    <span class="toggle-copy">
      <span class="toggle-label">Password-protect archive</span>
      <span class="toggle-desc">Optional outer seal on the backup file</span>
    </span>
    <span class="switch" class:on={protectExport}>
      <input type="checkbox" bind:checked={protectExport} />
      <span class="switch-track" aria-hidden="true">
        <span class="switch-thumb"></span>
      </span>
    </span>
  </label>

  {#if protectExport}
    <label class="field">
      <span class="field-label">Archive password</span>
      <input
        class="text-input"
        type="password"
        autocomplete="new-password"
        bind:value={exportPassword}
        disabled={busy}
      />
    </label>
    <label class="field">
      <span class="field-label">Confirm password</span>
      <input
        class="text-input"
        type="password"
        autocomplete="new-password"
        bind:value={exportConfirm}
        disabled={busy}
      />
    </label>
  {/if}

  <div class="row-actions">
    <button
      type="button"
      class="control-btn primary"
      disabled={busy || !paths}
      onclick={() => void runExport()}
    >
      {busy ? 'Working…' : 'Export backup…'}
    </button>
  </div>
</section>

<section class="panel" aria-labelledby="backup-import-heading">
  <div class="panel-head">
    <h3 id="backup-import-heading">Import backup</h3>
    <p class="help">
      Replaces the <strong>current vault</strong> with an archive. This cannot be
      undone from onQ. You will need to unlock again after import.
    </p>
  </div>
  <div class="row-actions">
    <button
      type="button"
      class="control-btn danger"
      disabled={busy || !paths}
      onclick={() => void startImport()}
    >
      Import backup…
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
  title="Replace current vault?"
  description="All prompts and the search index at this vault path will be replaced by the backup. The app will lock afterward so you can unlock the restored vault."
  itemLabel={pendingArchive ?? ''}
  itemKind="backup archive"
  confirmLabel="Replace vault"
  busyLabel="Importing…"
  cancelLabel="Cancel"
  {busy}
  onConfirm={() => void confirmImport()}
  onCancel={cancelImport}
>
  {#if pendingSealed}
    <label class="field import-pw">
      <span class="field-label">Archive password</span>
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
  .path-list {
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .path-row {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .path-row dt {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--glass-text-faint);
  }
  .path-row dd {
    margin: 0;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }
  .path {
    flex: 1 1 12rem;
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
  .control-btn.slim {
    padding: 8px 12px;
    font-size: 12px;
  }
  .control-btn.danger {
    border-color: color-mix(in srgb, var(--glass-danger) 40%, var(--glass-border));
    background: color-mix(in srgb, var(--glass-danger) 12%, var(--glass-control-bg));
  }
  .control-btn.danger:hover:not(:disabled) {
    background: color-mix(in srgb, var(--glass-danger) 22%, var(--glass-control-bg));
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .field-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--glass-text-dim);
  }
  .text-input {
    appearance: none;
    border: 1px solid var(--glass-border);
    background: var(--glass-control-bg);
    color: var(--glass-text);
    border-radius: 10px;
    padding: 10px 12px;
    font: inherit;
    font-size: 13px;
  }
  .text-input:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .import-pw {
    margin-top: 12px;
  }
  /* Local mirrors of settings panel chrome so this section stays self-contained
     when composed; SettingsPage also defines these for sibling panels. */
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
  h3 {
    margin: 0 0 6px;
    font-size: 15px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .help {
    margin: 0;
    color: var(--glass-text-dim);
    font-size: 13px;
    line-height: 1.5;
  }
  .help code {
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 12px;
    padding: 1px 5px;
    border-radius: 5px;
    background: var(--glass-control-bg);
    border: 1px solid var(--glass-border);
  }
  .hint {
    margin: 0;
    font-size: 12px;
    color: var(--glass-text-dim);
  }
  .hint.status {
    color: var(--glass-text);
  }
  .error {
    margin: 0;
    color: var(--glass-danger);
    font-size: 13px;
  }
  .row-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
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
  }
  .control-btn.primary {
    border-color: color-mix(in srgb, var(--glass-periwinkle) 38%, var(--glass-border));
    background: color-mix(in srgb, var(--glass-accent) 22%, var(--glass-control-bg));
  }
  .control-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    cursor: pointer;
  }
  .toggle-copy {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .toggle-label {
    font-size: 13px;
    font-weight: 650;
  }
  .toggle-desc {
    font-size: 12px;
    color: var(--glass-text-dim);
  }
  .switch {
    position: relative;
    width: 44px;
    height: 26px;
    flex-shrink: 0;
  }
  .switch input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
    margin: 0;
  }
  .switch-track {
    display: block;
    width: 100%;
    height: 100%;
    border-radius: 999px;
    background: var(--glass-control-bg);
    border: 1px solid var(--glass-border);
    transition: background 0.15s ease;
  }
  .switch.on .switch-track {
    background: color-mix(in srgb, var(--glass-accent) 45%, var(--glass-control-bg));
    border-color: color-mix(in srgb, var(--glass-periwinkle) 40%, var(--glass-border));
  }
  .switch-thumb {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--glass-text);
    transition: transform 0.15s ease;
    pointer-events: none;
  }
  .switch.on .switch-thumb {
    transform: translateX(18px);
  }
  .mono {
    font-family: 'JetBrains Mono', ui-monospace, monospace;
  }
</style>
