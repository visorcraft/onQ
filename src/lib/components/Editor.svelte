<script lang="ts">
  import { onMount } from 'svelte';
  import { createPrompt, readPrompt, savePrompt, deletePrompt } from '$lib/api/prompts';
  import { listFolders } from '$lib/api/folders';
  import { lockPrompt, unlockPrompt } from '$lib/api/lock';
  import { listPromptHistory, restorePromptHistory, type HistoryEntry } from '$lib/api/history';
  import { suggestTagsForBody } from '$lib/api/tags';
  import { fly, fade } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  import ConfirmDialog from './primitives/ConfirmDialog.svelte';
  import { t, locale } from '$lib/i18n';

  let {
    /** Existing prompt id, or `null` for an unsaved draft (not written until Save). */
    id = null,
    /** Pre-filled project path when opening a new draft. */
    initialFolder = null,
    onClose,
  }: {
    id?: string | null;
    initialFolder?: string | null;
    onClose: () => void;
  } = $props();

  const isDraft = $derived(id == null);

  let title = $state('');
  let body = $state('');
  let folderInput = $state('');
  let tags = $state<string[]>([]);
  let tagsInput = $state('');
  let favorite = $state(false);
  let locked = $state(false);
  let charCount = $state(0);
  let loading = $state(true);
  let projectOptions = $state<string[]>([]);
  let errorMessage = $state<string | null>(null);

  // Display-only when locked: the user can no longer edit the body in place
  // because the authoritative copy lives in the encrypted `.enc` envelope.
  // Saving while locked would create a stale plaintext copy in the vault,
  // defeating the lock. We force `onClose()` after lock/unlock so the
  // palette reloads and the next `readPrompt` reflects the new state.
  let busy = $state(false);
  let confirmDeleteOpen = $state(false);
  let copied = $state(false);
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;
  let historyEntries = $state<HistoryEntry[]>([]);
  let showHistory = $state(false);
  let tagSuggestions = $state<string[]>([]);
  let previewMode = $state<'edit' | 'preview'>('edit');

  onMount(() => {
    // Suppress page/sidebar scrollbars while open. Native OS scrollbars can
    // composite above backdrop-filter layers (common on Linux/WebKit), so the
    // Library nav scrollbar would otherwise remain visible over this dialog.
    document.documentElement.classList.add('editor-open');

    void (async () => {
      try {
        const folders = await listFolders().catch(() => []);
        projectOptions = folders.map((f) => f.name).sort((a, b) => a.localeCompare(b));

        if (id == null) {
          // Unsaved draft — nothing hits the vault until Save.
          folderInput = initialFolder ?? '';
          if (initialFolder && !projectOptions.includes(initialFolder)) {
            projectOptions = [...projectOptions, initialFolder].sort((a, b) =>
              a.localeCompare(b),
            );
          }
          return;
        }

        const p = await readPrompt(id);
        title = p.title;
        folderInput = p.folder ?? '';
        tags = p.tags ?? [];
        tagsInput = tags.join(', ');
        favorite = p.favorite;
        locked = p.locked;
        charCount = p.char_count;
        body = p.body ?? '';
        if (p.folder && !projectOptions.includes(p.folder)) {
          projectOptions = [...projectOptions, p.folder].sort((a, b) => a.localeCompare(b));
        }
        void listPromptHistory(id)
          .then((entries) => {
            historyEntries = entries;
          })
          .catch(() => undefined);
        void suggestTagsForBody(body)
          .then((s) => {
            tagSuggestions = s.filter((t) => !tags.includes(t));
          })
          .catch(() => undefined);
      } catch (e) {
        errorMessage = e instanceof Error ? e.message : String(e);
      } finally {
        loading = false;
      }
    })();

    return () => {
      document.documentElement.classList.remove('editor-open');
      if (copiedTimer !== undefined) clearTimeout(copiedTimer);
    };
  });

  function parseFolder(): string | null {
    const raw = folderInput.trim();
    return raw ? raw : null;
  }

  function parseTags(): string[] {
    return tagsInput
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean);
  }

  async function save() {
    if (locked || busy) return;
    busy = true;
    errorMessage = null;
    try {
      let promptId = id;
      const resolvedTitle = title.trim() || 'Untitled';
      // Create only when the user explicitly saves a draft.
      if (promptId == null) {
        const created = await createPrompt(resolvedTitle, parseFolder());
        promptId = created.id;
      }
      await savePrompt({
        id: promptId,
        title: resolvedTitle,
        body,
        folder: parseFolder(),
        tags: parseTags(),
        favorite,
      });
      onClose();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function requestDelete() {
    if (busy || isDraft) return;
    confirmDeleteOpen = true;
  }

  async function confirmDelete() {
    if (id == null) return;
    busy = true;
    errorMessage = null;
    try {
      await deletePrompt(id);
      confirmDeleteOpen = false;
      onClose();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
      confirmDeleteOpen = false;
    } finally {
      busy = false;
    }
  }

  async function toggleLock() {
    if (busy || isDraft || id == null) return;
    busy = true;
    errorMessage = null;
    try {
      if (locked) {
        // Never call save_prompt while locked — body is sealed in `.enc`.
        await unlockPrompt(id);
      } else {
        // Persist current plaintext before sealing it.
        await savePrompt({
          id,
          title: title.trim() || 'Untitled',
          body,
          folder: parseFolder(),
          tags: parseTags(),
          favorite,
        });
        await lockPrompt(id);
      }
      onClose();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function onEditorKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && !confirmDeleteOpen) {
      event.stopPropagation();
      onClose();
    }
  }

  function toggleFavorite() {
    if (locked || busy) return;
    favorite = !favorite;
  }

  async function copyBody() {
    if (locked || busy) return;
    errorMessage = null;
    try {
      await navigator.clipboard.writeText(body);
      copied = true;
      if (copiedTimer !== undefined) clearTimeout(copiedTimer);
      copiedTimer = setTimeout(() => {
        copied = false;
        copiedTimer = undefined;
      }, 1500);
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<button
  type="button"
  class="editor-backdrop"
  aria-label={t('editor.closeEditor', undefined, $locale)}
  transition:fade={{ duration: 160 }}
  onclick={onClose}
></button>
<div
  class="editor glass-elevated spring"
  transition:fly={{ y: 16, duration: 260, easing: quintOut }}
  role="dialog"
  aria-modal="true"
  aria-label={isDraft
    ? t('editor.newPrompt', undefined, $locale)
    : t('editor.editPrompt', undefined, $locale)}
  tabindex="-1"
  onkeydown={onEditorKeydown}
>
  <div class="ambient" aria-hidden="true"></div>

  {#if loading}
    <div class="loading-state">
      <div class="loading-pulse" aria-hidden="true"></div>
      <p>Loading prompt…</p>
    </div>
  {:else}
    <header class="editor-head">
      <p class="eyebrow">{isDraft ? 'New prompt' : 'Prompt'}</p>
      <div class="title-row">
        <label class="title-field">
          <span class="field-label">Title</span>
          <input
            class="title field-input"
            bind:value={title}
            placeholder={t('editor.titlePlaceholder', undefined, $locale)}
            aria-label={t('editor.titleAria', undefined, $locale)}
            disabled={locked}
          />
        </label>
        <div class="head-actions">
          <button
            type="button"
            class="icon-chip"
            class:on={favorite}
            title={favorite
              ? t('editor.unfavorite', undefined, $locale)
              : t('editor.favorite', undefined, $locale)}
            aria-label={favorite
              ? t('editor.unfavorite', undefined, $locale)
              : t('editor.favorite', undefined, $locale)}
            aria-pressed={favorite}
            disabled={locked || busy}
            onclick={toggleFavorite}
          >
            <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden="true">
              {#if favorite}
                <path
                  d="M8 1.8l1.7 3.5 3.9.6-2.8 2.7.7 3.8L8 10.6 4.5 12.4l.7-3.8L2.4 5.9l3.9-.6L8 1.8z"
                  fill="currentColor"
                />
              {:else}
                <path
                  d="M8 2.4l1.4 2.9 3.2.5-2.3 2.2.5 3.2L8 9.6 5.2 11.2l.5-3.2L3.4 5.8l3.2-.5L8 2.4z"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.4"
                  stroke-linejoin="round"
                />
              {/if}
            </svg>
          </button>
          {#if !isDraft}
            <button
              type="button"
              class="icon-chip"
              class:locked
              title={locked ? 'Unlock' : 'Lock'}
              aria-label={locked
                ? t('editor.unlock', undefined, $locale)
                : t('editor.lock', undefined, $locale)}
              aria-pressed={locked}
              disabled={busy}
              onclick={() => void toggleLock()}
            >
              <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden="true">
                {#if locked}
                  <path
                    d="M5 7V5.2a3 3 0 0 1 6 0V7"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.4"
                    stroke-linecap="round"
                  />
                  <rect
                    x="3.5"
                    y="7"
                    width="9"
                    height="6.5"
                    rx="1.5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.4"
                  />
                {:else}
                  <path
                    d="M5 7V5.2a3 3 0 0 1 5.7-1.3"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.4"
                    stroke-linecap="round"
                  />
                  <rect
                    x="3.5"
                    y="7"
                    width="9"
                    height="6.5"
                    rx="1.5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.4"
                  />
                {/if}
              </svg>
            </button>
          {/if}
          <button
            type="button"
            class="icon-chip close"
            aria-label={t('common.close', undefined, $locale)}
            onclick={onClose}
          >
            <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
              <path
                d="M3.5 3.5l9 9M12.5 3.5l-9 9"
                stroke="currentColor"
                stroke-width="1.6"
                stroke-linecap="round"
              />
            </svg>
          </button>
        </div>
      </div>
    </header>

    <div class="meta-row">
      <label class="field">
        <span class="field-label">{t('editor.project', undefined, $locale)}</span>
        <input
          class="field-input"
          list="project-paths"
          bind:value={folderInput}
          placeholder={t('editor.projectPlaceholder', undefined, $locale)}
          aria-label={t('editor.project', undefined, $locale)}
          disabled={locked}
        />
        <datalist id="project-paths">
          {#each projectOptions as opt (opt)}
            <option value={opt}></option>
          {/each}
        </datalist>
      </label>
      <label class="field">
        <span class="field-label">{t('editor.tags', undefined, $locale)}</span>
        <input
          class="field-input"
          bind:value={tagsInput}
          placeholder={t('editor.tagsPlaceholder', undefined, $locale)}
          aria-label={t('editor.tags', undefined, $locale)}
          disabled={locked}
          onkeydown={(e) => {
            if (e.key === 'Tab' && tagSuggestions.length > 0) {
              e.preventDefault();
              const next = tagSuggestions[0];
              const current = parseTags();
              if (!current.includes(next)) {
                tagsInput = [...current, next].join(', ');
              }
              tagSuggestions = tagSuggestions.slice(1);
            } else if (e.key === 'Escape') {
              tagSuggestions = [];
            }
          }}
        />
        {#if tagSuggestions.length > 0}
          <p class="hint" role="status">
            {t('editor.suggestHint', { tags: tagSuggestions.join(', ') }, $locale)}
          </p>
        {/if}
      </label>
    </div>

    {#if !isDraft && historyEntries.length > 0}
      <div class="history-block">
        <button type="button" class="control-btn" onclick={() => (showHistory = !showHistory)}>
          {t('editor.history', undefined, $locale)} ({historyEntries.length})
        </button>
        {#if showHistory}
          <ul class="history-list">
            {#each historyEntries as h (h.path)}
              <li>
                <span class="mono">{h.timestamp}</span>
                <button
                  type="button"
                  class="control-btn"
                  disabled={locked || busy}
                  onclick={() =>
                    void (async () => {
                      if (!id) return;
                      busy = true;
                      try {
                        const p = await restorePromptHistory(id, h.path);
                        body = p.body ?? '';
                        charCount = body.length;
                        historyEntries = await listPromptHistory(id);
                      } catch (e) {
                        errorMessage = e instanceof Error ? e.message : String(e);
                      } finally {
                        busy = false;
                      }
                    })()}
                >
                  {t('editor.restore', undefined, $locale)}
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    {/if}

    <div class="body-shell" class:is-locked={locked}>
      {#if locked}
        <div class="lock-banner" role="status">
          <span class="lock-dot" aria-hidden="true"></span>
          {t('editor.lockedBanner', undefined, $locale)}
        </div>
      {/if}
      <div class="body-toolbar">
        <button
          type="button"
          class="control-btn"
          class:on={previewMode === 'edit'}
          onclick={() => (previewMode = 'edit')}
        >
          {t('editor.edit', undefined, $locale)}
        </button>
        <button
          type="button"
          class="control-btn"
          class:on={previewMode === 'preview'}
          onclick={() => (previewMode = 'preview')}
        >
          {t('editor.preview', undefined, $locale)}
        </button>
      </div>
      {#if previewMode === 'preview'}
        <div class="body preview-pane" aria-label={t('editor.previewAria', undefined, $locale)}>
          {#each body.split('\n') as line}
            <p>{line || '\u00a0'}</p>
          {/each}
        </div>
      {:else}
        <textarea
          class="body"
          bind:value={body}
          oninput={(e) => {
            charCount = (e.target as HTMLTextAreaElement).value.length;
            void suggestTagsForBody((e.target as HTMLTextAreaElement).value)
              .then((s) => {
                tagSuggestions = s.filter((t) => !parseTags().includes(t));
              })
              .catch(() => undefined);
          }}
          placeholder={locked ? '' : t('editor.bodyPlaceholder', undefined, $locale)}
          aria-label={t('editor.bodyAria', undefined, $locale)}
          disabled={locked}
        ></textarea>
      {/if}
    </div>

    <div class="meta">
      <span class="char-count">
        <span class="char-num">{charCount.toLocaleString()}</span>
        characters
      </span>
      {#if locked}
        <span class="lock-badge">Encrypted</span>
      {/if}
    </div>

    {#if errorMessage}
      <p class="error" role="alert">{errorMessage}</p>
    {/if}

    <footer class="actions">
      {#if isDraft}
        <span class="draft-hint">Not saved yet — cancel discards this draft.</span>
      {:else}
        <button type="button" class="btn ghost danger-text" onclick={requestDelete} disabled={busy}>
          {t('editor.delete', undefined, $locale)}
        </button>
      {/if}
      <div class="actions-right">
        <button type="button" class="btn ghost" onclick={onClose}
          >{t('common.cancel', undefined, $locale)}</button
        >
        <button
          type="button"
          class="btn ghost"
          onclick={() => void copyBody()}
          disabled={locked || busy}
          aria-label={copied
            ? t('editor.copied', undefined, $locale)
            : t('editor.copy', undefined, $locale)}
        >
          {copied
            ? t('editor.copiedAction', undefined, $locale)
            : t('editor.copyAction', undefined, $locale)}
        </button>
        <button
          type="button"
          class="btn primary"
          onclick={() => void save()}
          disabled={locked || busy}
        >
          {busy && !confirmDeleteOpen
            ? t('editor.saving', undefined, $locale)
            : t('editor.save', undefined, $locale)}
        </button>
      </div>
    </footer>
  {/if}
</div>

{#if !isDraft}
  <ConfirmDialog
    bind:open={confirmDeleteOpen}
    title={t('editor.deleteTitle', undefined, $locale)}
    description={t('editor.deleteDesc', undefined, $locale)}
    itemLabel={title || t('library.untitled', undefined, $locale)}
    itemKind="Prompt"
    confirmLabel={t('editor.deleteConfirm', undefined, $locale)}
    busy={busy && confirmDeleteOpen}
    onConfirm={confirmDelete}
  />
{/if}

<style>
  .editor-backdrop {
    position: fixed;
    inset: 0;
    /* Above app chrome (z-index 40) and Library sticky nav; match Modal. */
    z-index: 100;
    border: 0;
    padding: 0;
    margin: 0;
    background:
      radial-gradient(ellipse 50% 40% at 50% 40%, rgba(47, 111, 237, 0.12), transparent 70%),
      rgba(4, 8, 18, 0.72);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    cursor: default;
  }
  .editor {
    position: fixed;
    inset: 0;
    margin: auto;
    width: min(820px, 92vw);
    height: min(84vh, 740px);
    padding: 0;
    display: flex;
    flex-direction: column;
    z-index: 101;
    overflow: hidden;
    /* Solid surface so content never bleeds through. */
    background: var(--glass-dialog);
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
  }
  /* Hide scrollable ancestors' scrollbars while the editor is open (see onMount). */
  :global(html.editor-open),
  :global(html.editor-open body) {
    overflow: hidden;
  }
  :global(html.editor-open) :global(main.page-mode) {
    overflow: hidden;
  }
  :global(html.editor-open) :global(aside.sidebar) {
    overflow: hidden;
  }
  .ambient {
    position: absolute;
    top: -30%;
    right: -10%;
    width: 360px;
    height: 280px;
    border-radius: 50%;
    background: radial-gradient(circle, var(--glass-accent-glow), transparent 68%);
    pointer-events: none;
  }
  .loading-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    color: var(--glass-text-dim);
    font-size: 14px;
  }
  .loading-pulse {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: 2px solid var(--glass-border-strong);
    border-top-color: var(--glass-periwinkle);
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .loading-pulse {
      animation: none;
      border-top-color: var(--glass-border-strong);
    }
  }
  .editor-head {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 22px 24px 12px;
  }
  .eyebrow {
    margin: 0;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--glass-cyan);
  }
  .title-row {
    display: flex;
    align-items: flex-end;
    gap: 10px;
  }
  .title-field {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .field-label {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--glass-text-faint);
  }
  .title.field-input {
    font-size: 18px;
    font-weight: 650;
    letter-spacing: -0.02em;
    line-height: 1.3;
    /* Match icon-chip height so the row reads as one control strip. */
    min-height: 42px;
    padding-top: 10px;
    padding-bottom: 10px;
  }
  .head-actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
    align-items: center;
  }
  .icon-chip {
    width: 42px;
    height: 42px;
    display: grid;
    place-items: center;
    border-radius: 11px;
    border: 1px solid var(--glass-border);
    background: var(--glass-control-bg);
    color: var(--glass-text-dim);
    cursor: pointer;
    line-height: 1;
    transition:
      background var(--motion-duration) ease,
      color var(--motion-duration) ease,
      border-color var(--motion-duration) ease;
  }
  .icon-chip:hover:not(:disabled) {
    background: var(--glass-hover-strong);
    color: var(--glass-text);
  }
  .icon-chip.on {
    color: var(--glass-gold);
    border-color: color-mix(in srgb, var(--glass-gold) 40%, transparent);
    background: color-mix(in srgb, var(--glass-gold) 12%, transparent);
  }
  .icon-chip.locked {
    color: var(--glass-periwinkle);
    border-color: color-mix(in srgb, var(--glass-periwinkle) 40%, transparent);
    background: color-mix(in srgb, var(--glass-periwinkle) 12%, transparent);
  }
  .icon-chip:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .icon-chip:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .meta-row {
    position: relative;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    padding: 0 24px 12px;
  }
  @media (max-width: 640px) {
    .meta-row {
      grid-template-columns: 1fr;
    }
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .field-input {
    width: 100%;
    box-sizing: border-box;
    font-size: 13px;
    background: var(--glass-input);
    border: 1px solid var(--glass-border);
    border-radius: 11px;
    padding: 10px 12px;
    color: var(--glass-text);
    font: inherit;
    transition:
      border-color var(--motion-duration) ease,
      box-shadow var(--motion-duration) ease;
  }
  .field-input::placeholder {
    color: var(--glass-text-faint);
  }
  .field-input:hover:not(:disabled) {
    border-color: var(--glass-border-strong);
  }
  .field-input:focus-visible {
    outline: none;
    border-color: color-mix(in srgb, var(--glass-periwinkle) 60%, transparent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--glass-periwinkle) 18%, transparent);
  }
  .body-shell {
    position: relative;
    flex: 1;
    min-height: 0;
    margin: 0 24px;
    display: flex;
    flex-direction: column;
    border-radius: 14px;
    border: 1px solid var(--glass-border);
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.03), transparent 40%),
      var(--glass-input);
    overflow: hidden;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
  }
  .body-shell.is-locked {
    border-color: color-mix(in srgb, var(--glass-periwinkle) 30%, var(--glass-border));
  }
  .lock-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    font-size: 12px;
    font-weight: 600;
    color: var(--glass-periwinkle);
    background: color-mix(in srgb, var(--glass-periwinkle) 10%, transparent);
    border-bottom: 1px solid color-mix(in srgb, var(--glass-periwinkle) 22%, transparent);
  }
  .lock-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--glass-periwinkle);
    box-shadow: 0 0 10px var(--glass-periwinkle);
  }
  .body {
    flex: 1;
    min-height: 0;
    width: 100%;
    box-sizing: border-box;
    background: transparent;
    border: 0;
    padding: 14px 16px;
    color: var(--glass-text);
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 13px;
    line-height: 1.55;
    resize: none;
    outline: none;
  }
  .body::placeholder {
    color: var(--glass-text-faint);
  }
  .meta {
    position: relative;
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: var(--glass-text-dim);
    font-size: 12px;
    gap: 12px;
    padding: 10px 28px 0;
  }
  .char-count {
    display: inline-flex;
    align-items: baseline;
    gap: 6px;
  }
  .char-num {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    color: var(--glass-text);
  }
  .lock-badge {
    display: inline-flex;
    align-items: center;
    padding: 3px 10px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--glass-periwinkle);
    border: 1px solid color-mix(in srgb, var(--glass-periwinkle) 35%, transparent);
    background: color-mix(in srgb, var(--glass-periwinkle) 12%, transparent);
  }
  .error {
    margin: 8px 24px 0;
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid var(--glass-danger-border);
    background: var(--glass-danger-bg);
    color: var(--glass-danger);
    font-size: 13px;
  }
  .actions {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 16px 24px 20px;
    margin-top: 4px;
  }
  .actions-right {
    display: flex;
    gap: 8px;
    margin-left: auto;
  }
  .draft-hint {
    font-size: 12px;
    color: var(--glass-text-faint);
  }
  .btn {
    appearance: none;
    border-radius: 11px;
    padding: 10px 16px;
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    border: 1px solid transparent;
    transition:
      background var(--motion-duration) ease,
      border-color var(--motion-duration) ease,
      box-shadow var(--motion-duration) ease;
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .btn:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .btn.ghost {
    background: transparent;
    border-color: var(--glass-border);
    color: var(--glass-text-dim);
  }
  .btn.ghost:hover:not(:disabled) {
    background: var(--glass-hover-strong);
    color: var(--glass-text);
  }
  .btn.danger-text {
    border-color: transparent;
    color: var(--glass-danger);
  }
  .btn.danger-text:hover:not(:disabled) {
    background: var(--glass-danger-bg);
    border-color: var(--glass-danger-border);
    color: var(--glass-danger);
  }
  .btn.primary {
    background: color-mix(in srgb, var(--glass-accent) 22%, var(--glass-control-bg));
    border-color: color-mix(in srgb, var(--glass-periwinkle) 38%, var(--glass-border));
    color: var(--glass-text);
    min-width: 96px;
    box-shadow: none;
  }
  .btn.primary:hover:not(:disabled) {
    background: color-mix(in srgb, var(--glass-accent) 32%, var(--glass-control-bg));
    border-color: color-mix(in srgb, var(--glass-periwinkle) 55%, var(--glass-border));
  }
  input:disabled,
  textarea:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
</style>
