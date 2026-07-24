<script lang="ts">
  import { t, locale } from '$lib/i18n';

  let {
    title = $bindable(''),
    isDraft,
    favorite,
    locked,
    busy,
    onToggleFavorite,
    onToggleLock,
    onClose,
  }: {
    title?: string;
    isDraft: boolean;
    favorite: boolean;
    locked: boolean;
    busy: boolean;
    onToggleFavorite: () => void;
    onToggleLock: () => void;
    onClose: () => void;
  } = $props();
</script>

<header class="editor-head">
  <div class="head-top">
    <div class="head-text">
      <h2 class="head-title">
        {isDraft ? t('editor.newPrompt', undefined, $locale) : t('editor.promptHeading', undefined, $locale)}
      </h2>
      <p class="head-sub">
        {isDraft
          ? t('editor.newTemplateSubtitle', undefined, $locale)
          : t('editor.editTemplateSubtitle', undefined, $locale)}
      </p>
    </div>
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
        onclick={onToggleFavorite}
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
          onclick={onToggleLock}
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
</header>

<style>
  .editor-head {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 22px 24px 12px;
  }
  .head-top {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }
  .head-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .head-title {
    margin: 0;
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--glass-text);
  }
  .head-sub {
    margin: 0;
    font-size: 13px;
    color: var(--glass-text-dim);
  }
  .title-field {
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
  .title.field-input {
    font-size: 18px;
    font-weight: 650;
    letter-spacing: -0.02em;
    line-height: 1.3;
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
  input:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
</style>
