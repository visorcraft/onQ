<script lang="ts">
  import { onMount } from 'svelte';
  import { getVaultAuthMode, retrieveVaultKey } from '$lib/api/vault';
  import { listPrompts, setPromptFavorite } from '$lib/api/prompts';
  import type { PromptSummary } from '$lib/types/prompt';
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
  import {
    createSmartFolder,
    deleteSmartFolder,
    listSmartFolders,
    type SmartFolder,
  } from '$lib/api/smartFolders';

  let {
    onBack,
    onOpenAbout,
  }: {
    onBack: () => void;
    onOpenAbout: () => void;
  } = $props();

  type SectionId =
    | 'general'
    | 'search'
    | 'vault'
    | 'projects'
    | 'favorites'
    | 'updates'
    | 'about';

  const metaKeyLabel = metaModifierLabel();
  const sections: { id: SectionId; label: string }[] = [
    { id: 'general', label: 'General' },
    { id: 'search', label: 'Search' },
    { id: 'vault', label: 'Vault & security' },
    { id: 'projects', label: 'Projects' },
    { id: 'favorites', label: 'Favorites' },
    { id: 'updates', label: 'Updates' },
    { id: 'about', label: 'About' },
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

  let prompts = $state<PromptSummary[]>([]);
  let smartFolders = $state<SmartFolder[]>([]);
  let orgError = $state<string | null>(null);
  let newProjectName = $state('');
  let newSmartName = $state('');
  let newSmartDsl = $state('favorite:true');
  let busyId = $state<string | null>(null);

  const favorites = $derived(prompts.filter((p) => p.favorite));
  const projects = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const p of prompts) {
      const key = (p.folder ?? '').trim() || 'Unfiled';
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
    return [...counts.entries()]
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => a.name.localeCompare(b.name));
  });

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
    void refreshOrg();
  });

  async function refreshOrg() {
    orgError = null;
    try {
      const [p, sf] = await Promise.all([listPrompts(), listSmartFolders()]);
      prompts = p;
      smartFolders = sf;
    } catch (e) {
      orgError = e instanceof Error ? e.message : String(e);
    }
  }

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

  async function unfavorite(p: PromptSummary) {
    busyId = p.id;
    orgError = null;
    try {
      const full = await setPromptFavorite(p.id, false);
      prompts = prompts.map((x) => (x.id === full.id ? { ...x, favorite: false } : x));
    } catch (e) {
      orgError = e instanceof Error ? e.message : String(e);
    } finally {
      busyId = null;
    }
  }

  async function assignProject(name: string) {
    // Create a "project" by renaming unfiled isn't enough — projects are
    // folder labels. Creating a smart folder named after the project is the
    // search-oriented way to pin a workspace filter.
    if (!name.trim()) return;
    orgError = null;
    try {
      const folder = name.trim();
      await createSmartFolder(folder, `folder:"${folder.replace(/"/g, '')}"`);
      newProjectName = '';
      await refreshOrg();
    } catch (e) {
      orgError = e instanceof Error ? e.message : String(e);
    }
  }

  async function addSmartFolder() {
    if (!newSmartName.trim() || !newSmartDsl.trim()) return;
    orgError = null;
    try {
      await createSmartFolder(newSmartName.trim(), newSmartDsl.trim());
      newSmartName = '';
      await refreshOrg();
    } catch (e) {
      orgError = e instanceof Error ? e.message : String(e);
    }
  }

  async function removeSmart(id: string) {
    orgError = null;
    try {
      await deleteSmartFolder(id);
      await refreshOrg();
    } catch (e) {
      orgError = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<div class="settings-page">
  <header class="settings-top">
    <button type="button" class="btn-ghost" onclick={onBack}>← Back</button>
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
      {:else if active === 'projects'}
        <section class="panel">
          <h2>Projects</h2>
          <p class="help">
            Projects are folder labels on prompts — search-oriented workspaces
            you can filter in the palette. Smart folders pin reusable DSL
            queries (favorites, tags, folders).
          </p>
          {#if orgError}<p class="error" role="alert">{orgError}</p>{/if}

          <h3 class="subhead">Folder projects</h3>
          <ul class="list">
            {#each projects as proj (proj.name)}
              <li>
                <span class="list-title">{proj.name}</span>
                <span class="list-meta">{proj.count} prompts</span>
              </li>
            {:else}
              <li class="empty">No folder labels yet — assign folders when editing prompts.</li>
            {/each}
          </ul>
          <div class="inline-form">
            <input
              type="text"
              placeholder="New project (smart folder for folder:Name)"
              bind:value={newProjectName}
            />
            <button
              type="button"
              class="control-btn"
              onclick={() => void assignProject(newProjectName)}
              disabled={!newProjectName.trim()}
            >
              Add smart project
            </button>
          </div>

          <h3 class="subhead">Smart folders</h3>
          <ul class="list">
            {#each smartFolders as sf (sf.id)}
              <li>
                <div>
                  <div class="list-title">{sf.name}</div>
                  <div class="list-meta mono">{sf.query_dsl}</div>
                </div>
                <button
                  type="button"
                  class="btn-ghost sm"
                  onclick={() => void removeSmart(sf.id)}>Delete</button
                >
              </li>
            {:else}
              <li class="empty">No smart folders yet.</li>
            {/each}
          </ul>
          <div class="inline-form stacked">
            <input type="text" placeholder="Name" bind:value={newSmartName} />
            <input type="text" placeholder="DSL e.g. favorite:true" bind:value={newSmartDsl} />
            <button type="button" class="control-btn" onclick={() => void addSmartFolder()}
              >Create smart folder</button
            >
          </div>
        </section>
      {:else if active === 'favorites'}
        <section class="panel">
          <h2>Favorites</h2>
          <p class="help">
            Starred prompts surface in search and smart folders. Manage them
            here without leaving Settings.
          </p>
          {#if orgError}<p class="error" role="alert">{orgError}</p>{/if}
          <ul class="list">
            {#each favorites as fav (fav.id)}
              <li>
                <div>
                  <div class="list-title">{fav.title}</div>
                  <div class="list-meta">
                    {fav.folder || 'Unfiled'}
                    {#if fav.tags?.length}
                      · {fav.tags.join(', ')}
                    {/if}
                  </div>
                </div>
                <button
                  type="button"
                  class="btn-ghost sm"
                  disabled={busyId === fav.id}
                  onclick={() => void unfavorite(fav)}
                >
                  Unfavorite
                </button>
              </li>
            {:else}
              <li class="empty">No favorites yet. Star prompts from the editor or palette.</li>
            {/each}
          </ul>
          <button type="button" class="btn-ghost sm" onclick={() => void refreshOrg()}
            >Refresh</button
          >
        </section>
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
            <span>Opt in to beta releases</span>
          </label>
        </section>
      {:else if active === 'about'}
        <section class="panel">
          <h2>About onQ</h2>
          <p class="help">
            Version, licenses, third-party credits, and runtime attributions.
          </p>
          <button type="button" class="control-btn" onclick={onOpenAbout}>Open About…</button>
        </section>
      {/if}

      {#if errorMessage}
        <p class="error" role="alert">{errorMessage}</p>
      {/if}
    </div>
  </div>
</div>

<style>
  .settings-page {
    box-sizing: border-box;
    width: min(1100px, 100%);
    margin: 0 auto;
    padding: 20px 20px 48px;
    color: var(--glass-text);
  }
  .settings-top {
    display: flex;
    gap: 14px;
    align-items: flex-start;
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
  .help code,
  .mono {
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
    background: rgba(12, 16, 26, 0.9);
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
    color: #7ee0d0;
    background: rgba(80, 220, 200, 0.12);
  }
  .nav-item:hover {
    background: rgba(255, 255, 255, 0.04);
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
    background: rgba(16, 22, 34, 0.92);
  }
  .control-btn,
  .btn-ghost {
    appearance: none;
    border: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.04);
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
  .btn-ghost {
    border-radius: 999px;
  }
  .btn-ghost.sm {
    padding: 6px 10px;
    font-size: 12px;
  }
  .control-btn:hover,
  .btn-ghost:hover {
    background: rgba(255, 255, 255, 0.08);
  }
  .control-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .hint {
    margin: 8px 0 0;
    font-size: 12px;
    color: var(--glass-text-dim);
  }
  .error {
    margin: 8px 0 0;
    color: #ffb4b4;
    font-size: 13px;
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
    background: rgba(120, 163, 255, 0.1);
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
  }
  textarea,
  input[type='text'] {
    width: 100%;
    box-sizing: border-box;
    border-radius: 10px;
    border: 1px solid var(--glass-border);
    background: rgba(10, 14, 22, 0.9);
    color: var(--glass-text);
    padding: 10px 12px;
    font: inherit;
    margin-bottom: 8px;
  }
  .list {
    list-style: none;
    margin: 0 0 12px;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .list li {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: center;
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.03);
  }
  .list-title {
    font-weight: 600;
    font-size: 13px;
  }
  .list-meta {
    font-size: 12px;
    color: var(--glass-text-dim);
  }
  .list .empty {
    color: var(--glass-text-dim);
    font-size: 13px;
  }
  .subhead {
    margin: 16px 0 8px;
    font-size: 13px;
    font-weight: 700;
    color: var(--glass-text-dim);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .inline-form {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .inline-form.stacked {
    flex-direction: column;
    align-items: stretch;
  }
  .inline-form input {
    margin: 0;
    flex: 1;
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
