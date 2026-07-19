<script lang="ts">
  import { onMount } from 'svelte';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import { getVaultAuthMode, retrieveVaultKey } from '$lib/api/vault';
  import {
    captureGlobalShortcut,
    globalShortcut,
    globalShortcutBackend,
    setGlobalShortcut,
    shortcutFromKeyboardEvent,
  } from '$lib/stores/globalShortcut';
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

  let { open = $bindable(false) }: { open?: boolean } = $props();

  // Tracks the most recent in-flight selection so we can show a one-line
  // error toast if the backend write fails. Cleared automatically when
  // the user picks another value or dismisses the modal.
  let errorMessage = $state<string | null>(null);
  let recordingShortcut = $state(false);
  let shortcutError = $state<string | null>(null);
  let authMode = $state<'keychain' | 'password' | null>(null);
  let recoveryPhrase = $state('');
  let encryptionKey = $state<string | null>(null);
  let keyError = $state<string | null>(null);
  let retrievingKey = $state(false);

  // Local shadow of the radio selection so we can drive `bind:group`
  // optimistically while a backend roundtrip is in flight, and roll
  // it back without touching the shared `embeddingQuant` store if the
  // write is rejected. Mirrors `$embeddingQuant` whenever the store
  // updates (mount, foreign event, etc.) so the radios stay in sync
  // without us having to write back on every change.
  let pendingQuant = $state<EmbeddingQuant>('binary');
  let lastSynced = $state<EmbeddingQuant | null>(null);

  // M7.2: mirror of `betaChannel` used to drive the checkbox's
  // checked state optimistically. The effect below re-syncs from the
  // store whenever it changes so an out-of-band write (e.g. another
  // panel) snaps the toggle into place.
  let pendingBeta = $state<boolean>(false);
  let lastSyncedBeta = $state<boolean | null>(null);

  $effect(() => {
    // Copy the store -> local on every change, but never during an
    // in-flight optimistic toggle (which would silently undo it).
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
    // The persisted choice lives in `app_state.embedding_quant`; load
    // it on mount so the panel shows the right radio on first open.
    void loadEmbeddingQuant().catch(() => undefined);
    // M7.2: load the beta opt-in so the toggle matches persisted state.
    void loadBetaChannel().catch(() => undefined);
    void getVaultAuthMode().then((mode) => (authMode = mode)).catch(() => undefined);
  });

  // Clear any leftover error toast when the modal (re)opens so the
  // user never sees a stale failure from their previous session.
  $effect(() => {
    if (open) {
      errorMessage = null;
      shortcutError = null;
      recordingShortcut = false;
      recoveryPhrase = '';
      encryptionKey = null;
      keyError = null;
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
      shortcutError =
        error instanceof Error ? error.message : String(error);
    }
  }

  async function startShortcutSetup() {
    shortcutError = null;
    recordingShortcut = true;
    if ($globalShortcutBackend !== 'linux-input') return;
    try {
      await captureGlobalShortcut();
    } catch (error) {
      shortcutError =
        error instanceof Error ? error.message : String(error);
    } finally {
      recordingShortcut = false;
    }
  }

  /**
   * Switch the active embedding quantization. The radio change is
   * driven through `bind:group` so the DOM updates immediately for
   * a snappy UX; on a backend rejection we roll `pendingQuant` back
   * to the last-synced store value so the radio snaps back to where
   * the persisted truth actually is.
   */
  async function pickQuant(next: EmbeddingQuant) {
    if (next === $embeddingQuant) return;
    errorMessage = null;
    const prev = $embeddingQuant;
    try {
      await setEmbeddingQuant(next);
    } catch (e) {
      // Rollback the optimistic flip — the store didn't move so the
      // effect above won't auto-repair; we have to do it by hand.
      pendingQuant = prev;
      errorMessage = e instanceof Error ? e.message : String(e);
    }
  }

  /**
   * M7.2: flip the beta-channel opt-in. The checkbox is bound to
   * `pendingBeta` so the UI updates instantly; on a backend rejection
   * we snap the toggle back so the user can retry instead of seeing
   * a persisted-but-not-recorded state.
   */
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
</script>

<Modal bind:open title="Settings">
  <section class="settings-section" aria-labelledby="shortcut-heading">
    <header class="section-head">
      <h3 id="shortcut-heading">Show onQ</h3>
      <p class="section-help">
        Choose a global shortcut. It restores onQ from the tray and
        opens the prompt palette.
      </p>
    </header>

    <button
      type="button"
      class:recording={recordingShortcut}
      class="shortcut-recorder"
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
      <p class="status hint" role="status">
        Press Ctrl, Alt, or Super plus another key. Escape cancels.
      </p>
    {/if}
    {#if shortcutError}
      <p class="status error" role="alert">
        Could not use that shortcut: {shortcutError}
      </p>
    {/if}
  </section>

  <hr class="section-divider" aria-hidden="true" />

  {#if authMode === 'keychain'}
    <section class="settings-section" aria-labelledby="key-heading">
      <header class="section-head">
        <h3 id="key-heading">Encryption key</h3>
        <p class="section-help">
          This no-password vault uses a key stored in your system keychain.
          Enter its recovery phrase only when you need to display that key.
        </p>
      </header>

      {#if encryptionKey}
        <label class="visually-hidden" for="encryption-key">Encryption key</label>
        <textarea id="encryption-key" readonly rows="3" value={encryptionKey}></textarea>
      {:else}
        <label class="visually-hidden" for="key-recovery-phrase">Recovery phrase</label>
        <textarea
          id="key-recovery-phrase"
          rows="3"
          bind:value={recoveryPhrase}
          placeholder="24-word recovery phrase"
          autocomplete="off"
          spellcheck="false"
        ></textarea>
        <button
          type="button"
          class="shortcut-recorder"
          disabled={retrievingKey || !recoveryPhrase.trim()}
          onclick={() => void showEncryptionKey()}
        >
          {retrievingKey ? 'Checking…' : 'Show encryption key'}
        </button>
      {/if}
      {#if keyError}<p class="status error" role="alert">{keyError}</p>{/if}
    </section>

    <hr class="section-divider" aria-hidden="true" />
  {/if}

  <section class="settings-section" aria-labelledby="embedding-heading">
    <header class="section-head">
      <h3 id="embedding-heading">Embedding quantization</h3>
      <p class="section-help">
        Trade-off between recall and search speed on the
        <code>prompts.embedding</code> ANN index. The active mode takes
        effect on the next search.
      </p>
    </header>

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
          Binary HNSW candidates + exact cosine rerank. Fast; ~95% recall;
          low memory. Recommended for vaults with many prompts.
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
          Full f32 cosine scan over the candidate set. Functionally correct
          (true cosine similarity) but slower than HNSW; use for small
          vaults or when you need maximum recall.
        </span>
      </label>
    </fieldset>

    {#if $rebuildingIndex}
      <p class="status rebuilding" role="status">Updating embedding index…</p>
    {:else}
      <p class="status hint">
        Note: the index itself is binary HNSW on disk until upstream
        MongrelDB exposes DROP/CREATE INDEX DDL — your choice is recorded
        and applies to the next index creation.
      </p>
    {/if}
  </section>

  <hr class="section-divider" aria-hidden="true" />

  <section class="settings-section" aria-labelledby="beta-heading">
    <header class="section-head">
      <h3 id="beta-heading">Beta channel</h3>
      <p class="section-help">
        Receive pre-release auto-updates. Beta builds ship new features
        earlier but may contain rough edges.
      </p>
    </header>

    <label class="toggle-row">
      <input
        type="checkbox"
        name="beta-channel"
        bind:checked={pendingBeta}
        onchange={(event) => flipBeta(event.currentTarget.checked)}
      />
      <span class="toggle-label">Opt in to beta releases</span>
    </label>

    <p class="status hint">
      Your choice is recorded in <code>app_state.beta_channel</code>. The
      auto-update channel split (<code>latest.json</code> vs
      <code>beta.json</code>) is a follow-up task — enabling the toggle
      persists the opt-in but does not yet redirect the updater, so
      production builds keep arriving until that work ships.
    </p>
  </section>

  {#if errorMessage}
    <p class="status error" role="alert">{errorMessage}</p>
  {/if}
</Modal>

<style>
  .settings-section {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .section-head h3 {
    margin: 0 0 4px;
    font-size: 15px;
    font-weight: 600;
    color: var(--glass-text);
  }
  .section-help {
    margin: 0;
    font-size: 12px;
    color: var(--glass-text-dim);
  }
  .section-help code {
    font-family: 'JetBrains Mono', monospace;
    font-size: 11px;
    background: rgba(127, 127, 127, 0.15);
    padding: 1px 5px;
    border-radius: 4px;
  }
  .radio-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 0;
    padding: 0;
    border: 0;
  }
  .radio-group[disabled] {
    opacity: 0.6;
    cursor: not-allowed;
  }
  .radio-card {
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-rows: auto auto;
    column-gap: 10px;
    row-gap: 2px;
    padding: 10px 12px;
    border: 1px solid var(--glass-border);
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.04);
    cursor: pointer;
    transition: background var(--motion-duration) ease,
      border-color var(--motion-duration) ease;
  }
  .radio-card:hover {
    background: rgba(255, 255, 255, 0.08);
  }
  .radio-card.selected {
    border-color: var(--glass-periwinkle);
    background: rgba(120, 163, 255, 0.10);
  }
  .radio-card input[type='radio'] {
    grid-row: 1 / span 2;
    align-self: start;
    margin-top: 3px;
    accent-color: var(--glass-periwinkle);
    cursor: pointer;
  }
  .radio-card:has(input:disabled) {
    cursor: not-allowed;
  }
  .radio-label {
    font-size: 14px;
    font-weight: 600;
    color: var(--glass-text);
  }
  .radio-desc {
    font-size: 12px;
    color: var(--glass-text-dim);
    line-height: 1.4;
  }
  .toggle-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border: 1px solid var(--glass-border);
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.04);
    cursor: pointer;
  }
  .toggle-row:hover {
    background: rgba(255, 255, 255, 0.08);
  }
  .toggle-row input[type='checkbox'] {
    accent-color: var(--glass-periwinkle);
    cursor: pointer;
    width: 16px;
    height: 16px;
  }
  .toggle-label {
    font-size: 14px;
    font-weight: 600;
    color: var(--glass-text);
  }
  .shortcut-recorder {
    min-height: 42px;
    padding: 9px 14px;
    border: 1px solid var(--glass-border);
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.04);
    color: var(--glass-text);
    font-family: 'JetBrains Mono', monospace;
    font-size: 13px;
    cursor: pointer;
  }
  .shortcut-recorder:hover,
  .shortcut-recorder.recording {
    border-color: var(--glass-periwinkle);
    background: rgba(120, 163, 255, 0.1);
  }
  .shortcut-recorder:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  textarea {
    box-sizing: border-box;
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--glass-border);
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.25);
    color: var(--glass-text);
    font: 12px/1.5 'JetBrains Mono', monospace;
    resize: vertical;
  }
  textarea:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .section-divider {
    border: 0;
    border-top: 1px solid var(--glass-border);
    margin: 4px 0;
  }
  .status {
    margin: 0;
    font-size: 12px;
    line-height: 1.4;
  }
  .status.rebuilding {
    color: var(--glass-periwinkle);
    font-style: italic;
  }
  .status.hint {
    color: var(--glass-text-dim);
  }
  .status.error {
    color: #ff9c9c;
  }
  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
