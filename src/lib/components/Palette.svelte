<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
  import { loadPrompts, createPrompt } from '$lib/stores/prompts';
  import { results as searchResults, runSearch, runMoreLikeThis } from '$lib/stores/search';
  import { recordOpen } from '$lib/api/recent';
  import { getAppSetting } from '$lib/api/settings';
  import {
    globalShortcutPressedEvent,
    matchesGlobalShortcut,
  } from '$lib/stores/globalShortcut';
  import { fly, fade } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  import Editor from './Editor.svelte';

  let open = $state(false);
  let query = $state('');
  let selectedId = $state<string | null>(null);
  let commandList = $state<HTMLDivElement>();
  let commandInput = $state<HTMLInputElement>();
  // Title of the prompt currently open in the editor. Lets the palette offer
  // a "More like this: <title>" command when the palette is opened while a prompt is
  // selected. Reset on editor close.
  let selectedTitle = $state<string | null>(null);
  // Recent-search queries from the encrypted `recent_searches` column.
  // Refreshed when the palette opens so the "Recent" group reflects the
  // latest stored queries.
  let recentSearches = $state<string[]>([]);

  async function refreshRecent() {
    try {
      const raw = await getAppSetting('recent_searches');
      const parsed = raw ? (JSON.parse(raw) as string[]) : [];
      recentSearches = Array.isArray(parsed) ? parsed.slice(0, 5) : [];
    } catch {
      recentSearches = [];
    }
  }

  function togglePalette() {
    open = !open;
    if (open) {
      loadPrompts();
      void refreshRecent();
      void tick().then(() => commandInput?.focus());
    }
  }

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
      if (e.key === 'Escape' && open) open = false;
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

  function onSelect(id: string, title: string) {
    selectedId = id;
    selectedTitle = title;
    open = false;
    // Record which prompt the user opened so the next app start can
    // re-open it. Best-effort — `recordOpen` swallows its own errors.
    void recordOpen(id);
  }

  async function onNew() {
    const p = await createPrompt('Untitled');
    selectedId = p.id;
    selectedTitle = p.title;
    open = false;
    void recordOpen(p.id);
  }

  function closeEditor() {
    selectedId = null;
    selectedTitle = null;
  }

  function onRecentSearch(text: string) {
    // Clicking a "Recent" item seeds the palette's input so the next
    // debounced search uses the same text. The backend will dedupe + move
    // it to the head of the recent column.
    query = text;
  }

  async function onMoreLikeThis() {
    // The trigger only appears when `selectedId` is set, but guard against
    // a race where the editor was closed between render and click.
    if (selectedId === null) return;
    open = false;
    // Don't await — the palette's already closed and `$searchResults`
    // updates when the backend returns. The user sees the "more like
    // this" hints fill in behind the editor.
    runMoreLikeThis(selectedId, 10);
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

{#if open}
  <button
    class="backdrop"
    onclick={() => (open = false)}
    transition:fade={{ duration: 160 }}
    aria-label="Close palette"
  ></button>
  <div
    class="palette glass-elevated spring"
    transition:fly={{ y: -20, duration: 240, easing: quintOut }}
    role="dialog"
    aria-label="Command palette"
    tabindex="-1"
    onkeydown={moveSelection}
  >
    <input
      class="palette-input"
      bind:this={commandInput}
      bind:value={query}
      placeholder="Search prompts, or type to create…"
    />
    <div class="palette-list" bind:this={commandList}>
      <button class="palette-item" type="button" onclick={onNew}>+ New prompt</button>
      {#if hits.length === 0}
        <div class="palette-empty">No results.</div>
      {/if}
      {#if selectedId !== null && selectedTitle !== null}
        <button class="palette-item" type="button" onclick={onMoreLikeThis}>
          More like this: {selectedTitle}
        </button>
      {/if}
      {#if query === '' && recentSearches.length > 0}
        <div class="palette-group-heading">Recent</div>
        {#each recentSearches as text (text)}
          <button
            class="palette-item"
            type="button"
            onclick={() => onRecentSearch(text)}
          >
            {text}
          </button>
        {/each}
      {/if}
      <div class="palette-group-heading">Prompts</div>
      {#each hits as h (h.id)}
        <button
          class="palette-item"
          type="button"
          onclick={() => onSelect(h.id, h.title)}
        >
          {h.title}
          {#if h.locked}<span aria-label="locked">🔒</span>{/if}
        </button>
      {/each}
    </div>
  </div>
{/if}

{#if selectedId}
  <Editor id={selectedId} onClose={closeEditor} />
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
  .palette-list {
    max-height: 50vh;
    overflow-y: auto;
    padding: 4px;
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
