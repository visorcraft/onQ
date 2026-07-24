<script lang="ts">
  import { t, locale } from '$lib/i18n';
  import TagChip from '../primitives/TagChip.svelte';

  let {
    tags,
    knownTags = [],
    suggestions = [],
    disabled = false,
    onChange,
    onSuggestionAccepted,
    onDismissSuggestions,
  }: {
    tags: string[];
    knownTags?: string[];
    suggestions?: string[];
    disabled?: boolean;
    onChange: (tags: string[]) => void; // eslint-disable-line no-unused-vars
    onSuggestionAccepted?: () => void;
    onDismissSuggestions?: () => void;
  } = $props();

  let draft = $state('');
  let dropdownOpen = $state(false);
  let rootEl: HTMLDivElement | undefined = $state(undefined);

  const filteredKnown = $derived(
    knownTags.filter(
      (k) =>
        !tags.some((tag) => tag.toLowerCase() === k.toLowerCase()) &&
        (draft.trim() === '' || k.toLowerCase().includes(draft.trim().toLowerCase())),
    ),
  );

  function commit(raw: string) {
    const parts = raw
      .split(',')
      .map((p) => p.trim())
      .filter(Boolean);
    const next = [...tags];
    for (const part of parts) {
      if (!next.some((tag) => tag.toLowerCase() === part.toLowerCase())) {
        next.push(part);
      }
    }
    if (next.length !== tags.length) onChange(next);
    draft = '';
  }

  function remove(tag: string) {
    onChange(tags.filter((existing) => existing !== tag));
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' || event.key === ',') {
      event.preventDefault();
      commit(draft);
    } else if (event.key === 'Backspace' && draft === '' && tags.length > 0) {
      event.preventDefault();
      onChange(tags.slice(0, -1));
    } else if (event.key === 'Tab' && suggestions.length > 0) {
      event.preventDefault();
      commit(suggestions[0]);
      onSuggestionAccepted?.();
    } else if (event.key === 'Escape') {
      // Never bubble Escape — the editor dialog closes on Escape.
      event.stopPropagation();
      if (dropdownOpen) {
        dropdownOpen = false;
      } else if (suggestions.length > 0) {
        onDismissSuggestions?.();
      }
    }
  }

  function onFocusOut(event: FocusEvent) {
    if (!rootEl?.contains(event.relatedTarget as Node | null)) {
      dropdownOpen = false;
    }
  }
</script>

<div class="tag-input" bind:this={rootEl} onfocusout={onFocusOut}>
  <div class="tag-box" class:is-disabled={disabled}>
    {#each tags as tag (tag)}
      <TagChip label={tag} removable={!disabled} onRemove={() => remove(tag)} />
    {/each}
    <input
      class="tag-draft"
      bind:value={draft}
      placeholder={tags.length === 0 ? t('editor.addTag', undefined, $locale) : ''}
      aria-label={t('editor.addTag', undefined, $locale)}
      {disabled}
      onkeydown={onKeydown}
      onfocus={() => (dropdownOpen = true)}
    />
    <button
      type="button"
      class="chevron"
      aria-label={t('editor.addTag', undefined, $locale)}
      aria-expanded={dropdownOpen}
      {disabled}
      onclick={() => (dropdownOpen = !dropdownOpen)}
    >
      <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
        <path
          d="M4 6l4 4 4-4"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
  </div>
  {#if dropdownOpen && filteredKnown.length > 0}
    <ul class="tag-dropdown">
      {#each filteredKnown as option (option)}
        <li>
          <button
            type="button"
            class="tag-option"
            onclick={() => {
              commit(option);
              dropdownOpen = false;
            }}
          >
            {option}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .tag-input {
    position: relative;
  }
  .tag-box {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    min-height: 42px;
    box-sizing: border-box;
    padding: 6px 8px;
    background: var(--glass-input);
    border: 1px solid var(--glass-border);
    border-radius: 11px;
    transition:
      border-color var(--motion-duration) ease,
      box-shadow var(--motion-duration) ease;
  }
  .tag-box:hover:not(.is-disabled) {
    border-color: var(--glass-border-strong);
  }
  .tag-box:focus-within {
    border-color: color-mix(in srgb, var(--glass-periwinkle) 60%, transparent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--glass-periwinkle) 18%, transparent);
  }
  .tag-box.is-disabled {
    opacity: 0.55;
  }
  .tag-draft {
    flex: 1;
    min-width: 90px;
    border: 0;
    background: transparent;
    padding: 4px 2px;
    color: var(--glass-text);
    font: inherit;
    font-size: 13px;
    outline: none;
  }
  .tag-draft::placeholder {
    color: var(--glass-text-faint);
  }
  .tag-draft:disabled {
    cursor: not-allowed;
  }
  .chevron {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--glass-text-dim);
    cursor: pointer;
  }
  .chevron:hover:not(:disabled) {
    background: var(--glass-hover-strong);
    color: var(--glass-text);
  }
  .chevron:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .chevron:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .tag-dropdown {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    right: 0;
    z-index: 5;
    margin: 0;
    padding: 4px;
    list-style: none;
    max-height: 180px;
    overflow-y: auto;
    background: var(--glass-dialog);
    border: 1px solid var(--glass-border);
    border-radius: 11px;
    box-shadow: 0 12px 32px rgba(2, 6, 16, 0.5);
  }
  .tag-option {
    display: block;
    width: 100%;
    text-align: left;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--glass-text);
    font: inherit;
    font-size: 13px;
    padding: 7px 10px;
    cursor: pointer;
  }
  .tag-option:hover {
    background: var(--glass-hover-strong);
  }
  .tag-option:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: -2px;
  }
</style>
