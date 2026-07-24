<script lang="ts">
  import { onMount } from 'svelte';
  import { createPrompt, readPrompt, savePrompt, deletePrompt, listPrompts } from '$lib/api/prompts';
  import { listFolders } from '$lib/api/folders';
  import { lockPrompt, unlockPrompt } from '$lib/api/lock';
  import { listPromptHistory, restorePromptHistory, type HistoryEntry } from '$lib/api/history';
  import { suggestTagsForBody } from '$lib/api/tags';
  import { fly, fade } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  import ConfirmDialog from './primitives/ConfirmDialog.svelte';
  import EditorHeader from './editor/EditorHeader.svelte';
  import TagInput from './editor/TagInput.svelte';
  import BodyEditor from './editor/BodyEditor.svelte';
  import EditorFooter from './editor/EditorFooter.svelte';
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
  let knownTags = $state<string[]>([]);
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

  const visibleSuggestions = $derived(
    tagSuggestions.filter(
      (s) => !tags.some((tag) => tag.toLowerCase() === s.toLowerCase()),
    ),
  );

  onMount(() => {
    // Suppress page/sidebar scrollbars while open. Native OS scrollbars can
    // composite above backdrop-filter layers (common on Linux/WebKit), so the
    // Library nav scrollbar would otherwise remain visible over this dialog.
    document.documentElement.classList.add('editor-open');

    void (async () => {
      try {
        const folders = await listFolders().catch(() => []);
        projectOptions = folders.map((f) => f.name).sort((a, b) => a.localeCompare(b));

        const summaries = await listPrompts().catch(() => []);
        const tagSet = new Set<string>();
        for (const p of summaries) {
          for (const tag of p.tags ?? []) {
            const trimmed = tag.trim();
            if (trimmed) tagSet.add(trimmed);
          }
        }
        knownTags = [...tagSet].sort((a, b) => a.localeCompare(b));

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
            tagSuggestions = s;
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
    return tags;
  }

  function acceptSuggestion() {
    tagSuggestions = tagSuggestions.slice(1);
  }

  function dismissSuggestions() {
    tagSuggestions = [];
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
    <EditorHeader
      bind:title
      {isDraft}
      {favorite}
      {locked}
      {busy}
      onToggleFavorite={toggleFavorite}
      onToggleLock={() => void toggleLock()}
      {onClose}
    />

    <div class="meta-row">
      <label class="field">
        <span class="field-label">{t('editor.project', undefined, $locale)}</span>
        <span class="input-icon-wrap">
          <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true" class="input-icon">
            <path
              d="M2 4.5A1.5 1.5 0 0 1 3.5 3h2.6a1.5 1.5 0 0 1 1.1.5l.9 1h4.4A1.5 1.5 0 0 1 14 6v5a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 11v-6.5z"
              fill="none"
              stroke="currentColor"
              stroke-width="1.3"
              stroke-linejoin="round"
            />
          </svg>
          <input
            class="field-input has-icon"
            list="project-paths"
            bind:value={folderInput}
            placeholder={t('editor.projectPlaceholder', undefined, $locale)}
            aria-label={t('editor.project', undefined, $locale)}
            disabled={locked}
          />
        </span>
        <p class="help">{t('editor.projectHelp', undefined, $locale)}</p>
        <datalist id="project-paths">
          {#each projectOptions as opt (opt)}
            <option value={opt}></option>
          {/each}
        </datalist>
      </label>
      <div class="field">
        <span class="field-label">{t('editor.tags', undefined, $locale)}</span>
        <TagInput
          {tags}
          {knownTags}
          suggestions={visibleSuggestions}
          disabled={locked}
          onChange={(next) => (tags = next)}
          onSuggestionAccepted={acceptSuggestion}
          onDismissSuggestions={dismissSuggestions}
        />
        <p class="help">{t('editor.tagsHelp', undefined, $locale)}</p>
        {#if visibleSuggestions.length > 0}
          <p class="hint" role="status">
            {t('editor.suggestHint', { tags: visibleSuggestions.join(', ') }, $locale)}
          </p>
        {/if}
      </div>
    </div>

    <BodyEditor
      {body}
      mode={previewMode}
      {locked}
      {busy}
      {isDraft}
      {historyEntries}
      {showHistory}
      onBodyInput={(value) => {
        body = value;
        charCount = value.length;
        void suggestTagsForBody(value)
          .then((s) => {
            tagSuggestions = s;
          })
          .catch(() => undefined);
      }}
      onModeChange={(mode) => (previewMode = mode)}
      onToggleHistory={() => (showHistory = !showHistory)}
      onRestoreHistory={(path) => {
        if (!id) return;
        busy = true;
        void (async () => {
          try {
            const p = await restorePromptHistory(id, path);
            body = p.body ?? '';
            charCount = body.length;
            historyEntries = await listPromptHistory(id);
          } catch (e) {
            errorMessage = e instanceof Error ? e.message : String(e);
          } finally {
            busy = false;
          }
        })();
      }}
    />

    {#if errorMessage}
      <p class="error" role="alert">{errorMessage}</p>
    {/if}

    <EditorFooter
      {charCount}
      {isDraft}
      {locked}
      {busy}
      saving={busy && !confirmDeleteOpen}
      {copied}
      onDelete={requestDelete}
      onCancel={onClose}
      onCopy={() => void copyBody()}
      onSave={() => void save()}
    />
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
  .field-label {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--glass-text-faint);
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
  .input-icon-wrap {
    position: relative;
    display: block;
  }
  .input-icon {
    position: absolute;
    left: 11px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--glass-text-faint);
    pointer-events: none;
  }
  .field-input.has-icon {
    padding-left: 34px;
  }
  .help {
    margin: 0;
    font-size: 12px;
    color: var(--glass-text-faint);
  }
  .hint {
    margin: 0;
    font-size: 12px;
    color: var(--glass-text-dim);
  }
  .hint::first-letter {
    font-weight: 700;
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
</style>
