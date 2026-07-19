<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { Command } from 'cmdk-sv';
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
  // Title of the prompt currently open in the editor. Lets the palette offer
  // a "More like this: <title>" command when ⌘K is opened while a prompt is
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
    }
  }

  onMount(() => {
    const handler = (e: KeyboardEvent) => {
      if (
        (e.metaKey || e.ctrlKey) &&
        e.key.toLowerCase() === 'k' &&
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
  >
    <Command.Root bind:value={query} loop>
      <Command.Input placeholder="Search prompts, or type to create…" autofocus />
      <Command.List>
        <Command.Empty>No results.</Command.Empty>
        <Command.Item onSelect={onNew} value="__new__">+ New prompt</Command.Item>
        {#if selectedId !== null && selectedTitle !== null}
          <Command.Item
            onSelect={onMoreLikeThis}
            value="__more_like_this__{selectedId}"
          >
            More like this: {selectedTitle}
          </Command.Item>
        {/if}
        {#if query === '' && recentSearches.length > 0}
          <Command.Group heading="Recent">
            {#each recentSearches as text (text)}
              <Command.Item onSelect={() => onRecentSearch(text)} value="__recent__{text}">
                {text}
              </Command.Item>
            {/each}
          </Command.Group>
        {/if}
        <Command.Group heading="Prompts">
          {#each hits as h (h.id)}
            <Command.Item onSelect={() => onSelect(h.id, h.title)} value={h.id}>
              {h.title}
              {#if h.locked}<span aria-label="locked">🔒</span>{/if}
            </Command.Item>
          {/each}
        </Command.Group>
      </Command.List>
    </Command.Root>
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
  :global([data-cmdk-input]) {
    width: 100%;
    padding: 12px 14px;
    background: transparent;
    border: 0;
    color: var(--glass-text);
    font-size: 15px;
    outline: none;
  }
  :global([data-cmdk-input])::placeholder {
    color: var(--glass-text-dim);
  }
  :global([data-cmdk-list]) {
    max-height: 50vh;
    overflow-y: auto;
    padding: 4px;
  }
  :global([data-cmdk-item]) {
    padding: 10px 12px;
    border-radius: 8px;
    color: var(--glass-text);
    cursor: pointer;
  }
  :global([data-cmdk-item][data-selected='true']) {
    background: rgba(91, 143, 255, 0.18);
  }
  :global([data-cmdk-empty]) {
    padding: 12px;
    color: var(--glass-text-dim);
    font-size: 13px;
  }
  :global([data-cmdk-group-heading]) {
    padding: 8px 12px 4px;
    color: var(--glass-text-dim);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
</style>
