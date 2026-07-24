<script lang="ts">
  import { t, locale } from '$lib/i18n';

  let {
    charCount,
    isDraft,
    locked,
    busy,
    saving,
    copied,
    onDelete,
    onCancel,
    onCopy,
    onSave,
  }: {
    charCount: number;
    isDraft: boolean;
    locked: boolean;
    busy: boolean;
    saving: boolean;
    copied: boolean;
    onDelete: () => void;
    onCancel: () => void;
    onCopy: () => void;
    onSave: () => void;
  } = $props();
</script>

<div class="meta">
  <span class="char-count">
    <span class="char-num">{charCount.toLocaleString()}</span>
    {t('editor.characters', undefined, $locale)}
  </span>
  <span class="meta-right">
    {#if locked}
      <span class="lock-badge">{t('editor.encryptedBadge', undefined, $locale)}</span>
    {/if}
    <span class="md-hint">
      {t('editor.markdownSupported', undefined, $locale)}
      <svg viewBox="0 0 16 16" width="13" height="13" aria-hidden="true">
        <circle
          cx="8"
          cy="8"
          r="6.4"
          fill="none"
          stroke="currentColor"
          stroke-width="1.3"
        />
        <path d="M8 7.2V11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        <circle cx="8" cy="5" r="0.9" fill="currentColor" />
      </svg>
    </span>
  </span>
</div>

<footer class="actions">
  {#if isDraft}
    <span class="draft-hint">{t('editor.draftHint', undefined, $locale)}</span>
  {:else}
    <button type="button" class="btn danger" onclick={onDelete} disabled={busy}>
      <svg viewBox="0 0 16 16" width="13" height="13" aria-hidden="true">
        <path
          d="M2.5 4h11M6.5 4V2.8a1 1 0 0 1 1-1h1a1 1 0 0 1 1 1V4M4 4l.7 8.4a1.4 1.4 0 0 0 1.4 1.3h3.8a1.4 1.4 0 0 0 1.4-1.3L12 4"
          fill="none"
          stroke="currentColor"
          stroke-width="1.3"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      {t('editor.delete', undefined, $locale)}
    </button>
  {/if}
  <div class="actions-right">
    <button type="button" class="btn ghost" onclick={onCancel}>
      {t('common.cancel', undefined, $locale)}
    </button>
    <button
      type="button"
      class="btn ghost"
      onclick={onCopy}
      disabled={locked || busy}
      aria-label={copied
        ? t('editor.copied', undefined, $locale)
        : t('editor.copy', undefined, $locale)}
    >
      <svg viewBox="0 0 16 16" width="13" height="13" aria-hidden="true">
        <rect
          x="5.5"
          y="5.5"
          width="8"
          height="8"
          rx="1.5"
          fill="none"
          stroke="currentColor"
          stroke-width="1.3"
        />
        <path
          d="M10.5 3.5v-0.2a1.3 1.3 0 0 0-1.3-1.3H3.8a1.3 1.3 0 0 0-1.3 1.3v5.4a1.3 1.3 0 0 0 1.3 1.3h0.2"
          fill="none"
          stroke="currentColor"
          stroke-width="1.3"
        />
      </svg>
      {copied
        ? t('editor.copiedAction', undefined, $locale)
        : t('editor.copyAction', undefined, $locale)}
    </button>
    <button type="button" class="btn primary" onclick={onSave} disabled={locked || busy}>
      <svg viewBox="0 0 16 16" width="13" height="13" aria-hidden="true">
        <path
          d="M3 2.5h7.5L13 5v8a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1v-10a1 1 0 0 1 1-1z"
          fill="none"
          stroke="currentColor"
          stroke-width="1.3"
          stroke-linejoin="round"
        />
        <path
          d="M5.5 2.5V6h5V2.5M5.5 13.5v-4h5v4"
          fill="none"
          stroke="currentColor"
          stroke-width="1.3"
          stroke-linejoin="round"
        />
      </svg>
      {saving ? t('editor.saving', undefined, $locale) : t('editor.save', undefined, $locale)}
    </button>
  </div>
</footer>

<style>
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
  .meta-right {
    display: inline-flex;
    align-items: center;
    gap: 10px;
  }
  .md-hint {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    color: var(--glass-text-faint);
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
    display: inline-flex;
    align-items: center;
    gap: 7px;
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
  .btn.danger {
    background: transparent;
    border-color: var(--glass-danger-border);
    color: var(--glass-danger);
  }
  .btn.danger:hover:not(:disabled) {
    background: var(--glass-danger-bg);
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
</style>
