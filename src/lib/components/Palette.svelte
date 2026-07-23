<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
  import { loadPrompts, prompts as promptsStore } from '$lib/stores/prompts';
  import { results as searchResults, runSearch, runMoreLikeThis } from '$lib/stores/search';
  import { recordOpen } from '$lib/api/recent';
  import { readPrompt } from '$lib/api/prompts';
  import { hideToTray } from '$lib/api/tray';
  import { minimizeOnCopy } from '$lib/stores/settings';
  import {
    globalShortcutPressedEvent,
    matchesGlobalShortcut,
  } from '$lib/stores/globalShortcut';
  import {
    getRecentPromptIds,
    pushRecentPromptId,
    RECENT_PROMPTS_CAP,
  } from '$lib/utils/recentPrompts';
  import type { PromptSummary } from '$lib/types/prompt';
  import { fly, fade } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  import Editor from './Editor.svelte';
  import {
    palette as paletteStore,
    closePalette as storeClosePalette,
    togglePalette as storeTogglePalette,
  } from '$lib/stores/palette.svelte';
  import { lockVaultNow } from '$lib/api/session';
  import { parseTemplateFields, renderTemplateBody } from '$lib/api/template';
  import { t, locale } from '$lib/i18n';
  import { highlightSnippet } from '$lib/utils/highlightSnippet';
  import {
    listPluginCommands,
    runPluginCommand,
    type PluginCommand,
  } from '$lib/api/plugins';

  let query = $state('');
  /** Existing prompt id, or `null` for an unsaved draft. */
  let editorId = $state<string | null | undefined>(undefined);
  let commandList = $state<HTMLDivElement>();
  let commandInput = $state<HTMLInputElement>();
  // Title of the prompt currently open in the editor. Lets the palette offer
  // a "More like this: <title>" command when the palette is opened while a prompt is
  // selected. Reset on editor close.
  let selectedTitle = $state<string | null>(null);
  let statusMessage = $state<string | null>(null);
  let copyingId = $state<string | null>(null);
  /** Bumped when local recent history changes so the list re-derives. */
  let recentEpoch = $state(0);
  let open = $derived(paletteStore.open);
  let pluginCommands = $state<PluginCommand[]>([]);

  function clearQuery() {
    query = '';
  }

  function closePalette() {
    storeClosePalette();
    clearQuery();
    statusMessage = null;
    copyingId = null;
  }

  function togglePalette() {
    storeTogglePalette();
  }

  // Rising-edge of `palette.open` (whether triggered by the keyboard
  // shortcut, the global OS shortcut, or the home card click) resets
  // transient state and refocuses the input. Decoupled from the
  // trigger so external callers don't have to know about the side
  // effects — `palette.svelte` owns the boolean, Palette owns the
  // input ref + prompt prefetch.
  let prevOpen = $state(false);
  $effect(() => {
    const isOpen = paletteStore.open;
    if (isOpen && !prevOpen) {
      prevOpen = true;
      clearQuery();
      statusMessage = null;
      recentEpoch += 1;
      loadPrompts();
      void listPluginCommands()
        .then((cmds) => {
          pluginCommands = cmds;
        })
        .catch(() => {
          pluginCommands = [];
        });
      void tick().then(() => commandInput?.focus());
    } else if (!isOpen && prevOpen) {
      prevOpen = false;
    }
  });

  onMount(() => {
    const handler = (e: KeyboardEvent) => {
      if (
        (e.metaKey || e.ctrlKey) &&
        e.key.toLowerCase() === 'q' &&
        !matchesGlobalShortcut(e)
      ) {
        e.preventDefault();
        togglePalette();
      }
      if (e.key === 'Escape' && open) closePalette();
    };
    const globalHandler = () => togglePalette();
    window.addEventListener('keydown', handler);
    window.addEventListener(globalShortcutPressedEvent, globalHandler);
    return () => {
      window.removeEventListener('keydown', handler);
      window.removeEventListener(globalShortcutPressedEvent, globalHandler);
    };
  });

  // Hybrid-search results from the backend. Empty when the embedder isn't
  // loaded (graceful degradation) or before the first keystroke settles.
  const hits = $derived($searchResults);
  const allPrompts = $derived($promptsStore);
  const hasQuery = $derived(query.trim().length > 0);

  /**
   * Recently used prompts for the empty-query palette. Prefer local history
   * (copy/edit order); fall back to most-recently-updated vault prompts so a
   * fresh install still has a useful list.
   */
  const recentPrompts = $derived.by(() => {
    // Depend on epoch so pushRecentPromptId updates re-render.
    void recentEpoch;
    const byId = new Map(allPrompts.map((p) => [p.id, p]));
    const fromHistory: PromptSummary[] = [];
    for (const id of getRecentPromptIds()) {
      const p = byId.get(id);
      if (p) fromHistory.push(p);
    }
    if (fromHistory.length > 0) return fromHistory.slice(0, RECENT_PROMPTS_CAP);

    return [...allPrompts]
      .sort((a, b) => b.updated.localeCompare(a.updated))
      .slice(0, RECENT_PROMPTS_CAP);
  });

  function rememberPrompt(id: string) {
    pushRecentPromptId(id);
    recentEpoch += 1;
  }

  /** Primary action: copy prompt body to the clipboard, then close. */
  async function onCopy(id: string) {
    if (copyingId) return;
    copyingId = id;
    statusMessage = null;
    try {
      const p = await readPrompt(id);
      if (p.locked) {
        statusMessage = t('palette.unlockToCopy');
        return;
      }
      let body = p.body ?? '';
      const fields = await parseTemplateFields(body);
      if (fields.length > 0) {
        const values: Record<string, string> = {};
        for (const field of fields) {
          const def = field.default ?? '';
          const answer =
            typeof window !== 'undefined'
              ? window.prompt(`Value for {{${field.name}}}`, def)
              : def;
          if (answer === null) {
            statusMessage = t('palette.copyCancelled');
            return;
          }
          values[field.name] = answer;
        }
        body = await renderTemplateBody(body, values);
      }
      await navigator.clipboard.writeText(body);
      rememberPrompt(id);
      closePalette();
      // Honour the user's "minimize on copy" preference — fire-and-forget;
      // the user just clicked a result and the next action they probably
      // want is to switch back to the source app. The hide call is async
      // because the palette close transition is still running.
      if ($minimizeOnCopy) {
        void hideToTray();
      }
    } catch (e) {
      statusMessage = e instanceof Error ? e.message : String(e);
    } finally {
      copyingId = null;
    }
  }

  async function onLockVault() {
    try {
      const path = await lockVaultNow();
      closePalette();
      window.dispatchEvent(
        new CustomEvent('onq:vault-locked', { detail: { path } }),
      );
    } catch (e) {
      statusMessage = e instanceof Error ? e.message : String(e);
    }
  }

  /** Secondary action: open the edit modal. */
  function onEdit(id: string, title: string) {
    editorId = id;
    selectedTitle = title;
    rememberPrompt(id);
    closePalette();
    // Record which prompt the user opened so the next app start can
    // re-open it. Best-effort — `recordOpen` swallows its own errors.
    void recordOpen(id);
  }

  function onNew() {
    // Draft only — vault write happens when the user hits Save.
    editorId = null;
    selectedTitle = null;
    closePalette();
  }

  function closeEditor() {
    editorId = undefined;
    selectedTitle = null;
  }

  async function onMoreLikeThis() {
    // The trigger only appears when an existing prompt is open in the editor.
    if (editorId == null || editorId === undefined) return;
    const id = editorId;
    closePalette();
    // Don't await — the palette's already closed and `$searchResults`
    // updates when the backend returns. The user sees the "more like
    // this" hints fill in behind the editor.
    runMoreLikeThis(id, 10);
  }

  function moveSelection(event: KeyboardEvent) {
    if (!['ArrowDown', 'ArrowUp', 'Enter'].includes(event.key)) return;
    if (!commandList) return;
    const items = Array.from(commandList.querySelectorAll<HTMLButtonElement>('.palette-item'));
    if (items.length === 0) return;
    if (event.key === 'Enter') {
      if (event.target instanceof HTMLInputElement) {
        event.preventDefault();
        items[0].click();
      }
      return;
    }
    event.preventDefault();
    const current = items.indexOf(document.activeElement as HTMLButtonElement);
    const direction = event.key === 'ArrowDown' ? 1 : -1;
    const next =
      current < 0
        ? direction === 1
          ? 0
          : items.length - 1
        : (current + direction + items.length) % items.length;
    items[next].focus();
  }

  // Drive the debounced search whenever the query string changes.
  // `untrack` keeps the effect from re-firing when `$searchResults` updates,
  // which would otherwise cause an infinite loop.
  $effect(() => {
    const q = query;
    untrack(() => {
      runSearch({ text: q, limit: 50 });
    });
  });
</script>

{#snippet promptRow(
  id: string,
  title: string,
  locked: boolean,
  favorite: boolean,
  snippet: string = '',
)}
  <div class="palette-row">
    <button
      class="palette-item palette-copy"
      type="button"
      title={t('palette.copyTitle', undefined, $locale)}
      disabled={copyingId === id}
      onclick={() => void onCopy(id)}
    >
      <span class="palette-main">
        <span class="palette-title">
          {#if favorite}
            <span
              class="favorite-star"
              aria-label={t('editor.favorite', undefined, $locale)}
              title={t('editor.favorite', undefined, $locale)}>★</span
            >
          {/if}
          {title}
          {#if locked}
            <svg
              class="lock-icon"
              viewBox="0 0 16 16"
              width="12"
              height="12"
              aria-label={t('common.locked', undefined, $locale)}
            >
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
            </svg>
          {/if}
        </span>
        {#if snippet}
          <!-- highlightSnippet escapes HTML before inserting <mark> tags -->
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          <span class="palette-snippet">{@html highlightSnippet(snippet, query)}</span>
        {/if}
      </span>
      <span class="palette-hint">{t('palette.copy', undefined, $locale)}</span>
    </button>
    <button
      class="palette-edit"
      type="button"
      title={t('palette.edit', undefined, $locale)}
      aria-label="{t('palette.edit', undefined, $locale)} {title}"
      onclick={() => onEdit(id, title)}
    >
      <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
        <path
          d="M11.5 2.5a1.4 1.4 0 0 1 2 2L5.8 12.2 3 13l.8-2.8L11.5 2.5Z"
          fill="none"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linejoin="round"
        />
        <path
          d="M10.2 3.8l2 2"
          fill="none"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linecap="round"
        />
      </svg>
    </button>
  </div>
{/snippet}

{#if open}
  <button
    type="button"
    class="backdrop"
    onclick={closePalette}
    transition:fade={{ duration: 160 }}
    aria-label={t('palette.close', undefined, $locale)}
  ></button>
  <div
    class="palette glass-elevated spring"
    transition:fly={{ y: -20, duration: 240, easing: quintOut }}
    role="dialog"
    aria-label={t('app.openPalette', undefined, $locale)}
    tabindex="-1"
    onkeydown={moveSelection}
  >
    <input
      class="palette-input"
      bind:this={commandInput}
      bind:value={query}
      placeholder={t('palette.placeholder', undefined, $locale)}
    />
    {#if statusMessage}
      <p class="palette-status" role="status">{statusMessage}</p>
    {/if}
    <div class="palette-list" bind:this={commandList}>
      <button class="palette-item" type="button" onclick={onNew}
        >{t('palette.newPrompt', undefined, $locale)}</button
      >
      {#if !hasQuery || query.toLowerCase().includes('lock')}
        <button class="palette-item" type="button" onclick={() => void onLockVault()}>
          {t('palette.lockVault', undefined, $locale)}
        </button>
      {/if}

      {#if pluginCommands.length > 0 && (!hasQuery || query.toLowerCase().includes('plugin') || pluginCommands.some((c) => c.name.toLowerCase().includes(query.toLowerCase())))}
        <div class="palette-group-heading">
          {t('palette.pluginCommands', undefined, $locale)}
        </div>
        {#each pluginCommands as cmd (cmd.id)}
          {#if !hasQuery || cmd.name.toLowerCase().includes(query.toLowerCase()) || query.toLowerCase().includes('plugin')}
            <button
              class="palette-item"
              type="button"
              onclick={() =>
                void runPluginCommand(cmd.id)
                  .then((msg) => {
                    statusMessage = msg;
                  })
                  .catch((e) => {
                    statusMessage = e instanceof Error ? e.message : String(e);
                  })}
            >
              {cmd.name}
              <span class="palette-hint">{cmd.pluginId}</span>
            </button>
          {/if}
        {/each}
      {/if}

      {#if editorId != null && selectedTitle !== null}
        <button class="palette-item" type="button" onclick={() => void onMoreLikeThis()}>
          More like this: {selectedTitle}
        </button>
      {/if}

      {#if !hasQuery}
        {#if recentPrompts.length > 0}
          <div class="palette-group-heading">
            {t('palette.recent', undefined, $locale)}
          </div>
          {#each recentPrompts as p (p.id)}
            {@render promptRow(
              p.id,
              p.title || t('library.untitled', undefined, $locale),
              p.locked,
              p.favorite,
              '',
            )}
          {/each}
        {/if}
        {#if hits.length > 0}
          <div class="palette-group-heading">
            {t('palette.similar', undefined, $locale)}
          </div>
          {#each hits as h (h.id)}
            {@render promptRow(
              h.id,
              h.title || t('library.untitled', undefined, $locale),
              h.locked,
              h.favorite,
              h.snippet ?? '',
            )}
          {/each}
        {/if}
      {:else}
        <div class="palette-group-heading">
          {t('palette.prompts', undefined, $locale)}
        </div>
        {#if hits.length === 0}
          <div class="palette-empty">{t('palette.noResults', undefined, $locale)}</div>
        {:else}
          {#each hits as h (h.id)}
            {@render promptRow(
              h.id,
              h.title || t('library.untitled', undefined, $locale),
              h.locked,
              h.favorite,
              h.snippet ?? '',
            )}
          {/each}
        {/if}
      {/if}
    </div>
  </div>
{/if}

{#if editorId !== undefined}
  {#key editorId ?? 'draft'}
    <Editor id={editorId} onClose={closeEditor} />
  {/key}
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(10, 14, 36, 0.6);
    border: 0;
    z-index: 10;
    cursor: default;
  }
  .palette {
    position: fixed;
    top: 20vh;
    left: 50%;
    transform: translateX(-50%);
    width: min(640px, 90vw);
    z-index: 11;
    padding: 12px;
  }
  .palette-input {
    width: 100%;
    padding: 12px 14px;
    background: transparent;
    border: 0;
    color: var(--glass-text);
    font-size: 15px;
    outline: none;
  }
  .palette-input::placeholder {
    color: var(--glass-text-dim);
  }
  .palette-status {
    margin: 0 12px 6px;
    padding: 8px 10px;
    border-radius: 8px;
    font-size: 12px;
    color: var(--glass-danger, #f07178);
    background: var(--glass-danger-bg, rgba(240, 113, 120, 0.12));
    border: 1px solid var(--glass-danger-border, rgba(240, 113, 120, 0.35));
  }
  .palette-list {
    max-height: 50vh;
    overflow-y: auto;
    padding: 4px;
  }
  .palette-main {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }
  .palette-snippet {
    font-size: 12px;
    line-height: 1.35;
    color: var(--glass-text-dim);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    text-align: left;
    width: 100%;
  }
  .palette-snippet :global(mark) {
    background: color-mix(in oklab, var(--accent, #7aa2ff) 35%, transparent);
    color: inherit;
    border-radius: 2px;
    padding: 0 1px;
  }
  .palette-row {
    display: flex;
    align-items: stretch;
    gap: 2px;
    border-radius: 8px;
  }
  .palette-row:hover,
  .palette-row:focus-within {
    background: rgba(91, 143, 255, 0.1);
  }
  .palette-item {
    display: block;
    width: 100%;
    padding: 10px 12px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--glass-text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .palette-item:focus-visible {
    background: rgba(91, 143, 255, 0.18);
  }
  .palette-copy {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    border-radius: 8px 0 0 8px;
  }
  .palette-copy:disabled {
    opacity: 0.6;
    cursor: wait;
  }
  .palette-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }
  .lock-icon {
    flex-shrink: 0;
    color: var(--glass-text-faint);
  }
  .favorite-star {
    color: var(--glass-gold, #f0c674);
    font-size: 12px;
    line-height: 1;
    flex-shrink: 0;
  }
  .palette-hint {
    flex-shrink: 0;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--glass-text-faint);
    opacity: 0;
    transition: opacity 0.12s ease;
  }
  .palette-row:hover .palette-hint,
  .palette-row:focus-within .palette-hint,
  .palette-copy:focus-visible .palette-hint {
    opacity: 1;
  }
  .palette-edit {
    flex-shrink: 0;
    width: 40px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 0 8px 8px 0;
    background: transparent;
    color: var(--glass-text-dim);
    cursor: pointer;
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }
  .palette-edit:hover {
    background: rgba(91, 143, 255, 0.16);
    color: var(--glass-text);
  }
  .palette-edit:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: -2px;
    color: var(--glass-text);
  }
  .palette-empty {
    padding: 12px;
    color: var(--glass-text-dim);
    font-size: 13px;
  }
  .palette-group-heading {
    padding: 8px 12px 4px;
    color: var(--glass-text-dim);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
</style>
