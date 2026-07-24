<script lang="ts">
  import { t, locale } from '$lib/i18n';
  import type { HistoryEntry } from '$lib/api/history';

  let {
    body,
    mode,
    locked,
    busy,
    isDraft,
    historyEntries,
    showHistory,
    onBodyInput,
    onModeChange,
    onToggleHistory,
    onRestoreHistory,
  }: {
    body: string;
    mode: 'edit' | 'preview';
    locked: boolean;
    busy: boolean;
    isDraft: boolean;
    historyEntries: HistoryEntry[];
    showHistory: boolean;
    onBodyInput: (value: string) => void; // eslint-disable-line no-unused-vars
    onModeChange: (mode: 'edit' | 'preview') => void; // eslint-disable-line no-unused-vars
    onToggleHistory: () => void;
    onRestoreHistory: (path: string) => void; // eslint-disable-line no-unused-vars
  } = $props();

  const lineNumbers = $derived(
    Array.from({ length: body.split('\n').length }, (_, i) => i + 1),
  );

  let gutterEl: HTMLDivElement | undefined = $state(undefined);

  function onTextareaScroll(event: Event) {
    if (gutterEl) {
      gutterEl.scrollTop = (event.target as HTMLTextAreaElement).scrollTop;
    }
  }
</script>

<div class="body-shell" class:is-locked={locked}>
  {#if locked}
    <div class="lock-banner" role="status">
      <span class="lock-dot" aria-hidden="true"></span>
      {t('editor.lockedBanner', undefined, $locale)}
    </div>
  {/if}
  <div class="body-toolbar">
    <div class="segmented">
      <button
        type="button"
        class="segment"
        class:on={mode === 'edit'}
        aria-pressed={mode === 'edit'}
        onclick={() => onModeChange('edit')}
      >
        <svg viewBox="0 0 16 16" width="13" height="13" aria-hidden="true">
          <path
            d="M6 4L2.5 8 6 12M10 4l3.5 4L10 12"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        {t('editor.editorTab', undefined, $locale)}
      </button>
      <button
        type="button"
        class="segment"
        class:on={mode === 'preview'}
        aria-pressed={mode === 'preview'}
        onclick={() => onModeChange('preview')}
      >
        <svg viewBox="0 0 16 16" width="13" height="13" aria-hidden="true">
          <path
            d="M1.5 8s2.5-4.5 6.5-4.5S14.5 8 14.5 8 12 12.5 8 12.5 1.5 8 1.5 8z"
            fill="none"
            stroke="currentColor"
            stroke-width="1.4"
            stroke-linejoin="round"
          />
          <circle cx="8" cy="8" r="2" fill="none" stroke="currentColor" stroke-width="1.4" />
        </svg>
        {t('editor.preview', undefined, $locale)}
      </button>
    </div>
    {#if !isDraft && historyEntries.length > 0}
      <button
        type="button"
        class="history-btn"
        aria-expanded={showHistory}
        onclick={onToggleHistory}
      >
        <svg viewBox="0 0 16 16" width="13" height="13" aria-hidden="true">
          <circle
            cx="8"
            cy="8"
            r="6.2"
            fill="none"
            stroke="currentColor"
            stroke-width="1.4"
          />
          <path
            d="M8 4.8V8l2.2 1.4"
            fill="none"
            stroke="currentColor"
            stroke-width="1.4"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        {t('editor.history', undefined, $locale)} ({historyEntries.length})
        <svg
          viewBox="0 0 16 16"
          width="12"
          height="12"
          aria-hidden="true"
          class="history-chevron"
          class:open={showHistory}
        >
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
    {/if}
  </div>
  {#if showHistory && !isDraft && historyEntries.length > 0}
    <ul class="history-list">
      {#each historyEntries as h (h.path)}
        <li>
          <span class="mono">{h.timestamp}</span>
          <button
            type="button"
            class="restore-btn"
            disabled={locked || busy}
            onclick={() => onRestoreHistory(h.path)}
          >
            {t('editor.restore', undefined, $locale)}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
  {#if mode === 'preview'}
    <div class="body preview-pane" aria-label={t('editor.previewAria', undefined, $locale)}>
      {#each body.split('\n') as line}
        <p>{line || ' '}</p>
      {/each}
    </div>
  {:else}
    <div class="code-wrap">
      <div class="gutter" bind:this={gutterEl} aria-hidden="true">
        {#each lineNumbers as n (n)}
          <span>{n}</span>
        {/each}
      </div>
      <textarea
        class="body code"
        value={body}
        oninput={(e) => onBodyInput((e.target as HTMLTextAreaElement).value)}
        onscroll={onTextareaScroll}
        placeholder={locked ? '' : t('editor.bodyPlaceholder', undefined, $locale)}
        aria-label={t('editor.bodyAria', undefined, $locale)}
        disabled={locked}
      ></textarea>
    </div>
  {/if}
</div>

<style>
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
  .body-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 8px 10px;
    border-bottom: 1px solid var(--glass-border);
  }
  .segmented {
    display: inline-flex;
    gap: 2px;
    padding: 2px;
    border-radius: 9px;
    background: var(--glass-control-bg);
    border: 1px solid var(--glass-border);
  }
  .segment {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--glass-text-dim);
    font: inherit;
    font-size: 12px;
    font-weight: 600;
    padding: 5px 12px;
    cursor: pointer;
    transition:
      background var(--motion-duration) ease,
      color var(--motion-duration) ease;
  }
  .segment:hover {
    color: var(--glass-text);
  }
  .segment.on {
    background: color-mix(in srgb, var(--glass-accent) 26%, var(--glass-control-bg));
    color: var(--glass-text);
  }
  .segment:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .history-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border: 1px solid var(--glass-border);
    border-radius: 9px;
    background: var(--glass-control-bg);
    color: var(--glass-text-dim);
    font: inherit;
    font-size: 12px;
    font-weight: 600;
    padding: 5px 10px;
    cursor: pointer;
    transition:
      background var(--motion-duration) ease,
      color var(--motion-duration) ease;
  }
  .history-btn:hover {
    background: var(--glass-hover-strong);
    color: var(--glass-text);
  }
  .history-btn:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .history-chevron {
    transition: transform var(--motion-duration) ease;
  }
  .history-chevron.open {
    transform: rotate(180deg);
  }
  .history-list {
    margin: 0;
    padding: 6px 14px;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 140px;
    overflow-y: auto;
    border-bottom: 1px solid var(--glass-border);
  }
  .history-list li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    font-size: 12px;
    color: var(--glass-text-dim);
  }
  .mono {
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-variant-numeric: tabular-nums;
  }
  .restore-btn {
    border: 1px solid var(--glass-border);
    border-radius: 7px;
    background: transparent;
    color: var(--glass-text-dim);
    font: inherit;
    font-size: 12px;
    padding: 3px 10px;
    cursor: pointer;
  }
  .restore-btn:hover:not(:disabled) {
    background: var(--glass-hover-strong);
    color: var(--glass-text);
  }
  .restore-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
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
  .preview-pane {
    overflow-y: auto;
    font-family: inherit;
  }
  .preview-pane p {
    margin: 0 0 2px;
  }
  .code-wrap {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: stretch;
  }
  .gutter {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    padding: 14px 8px 14px 14px;
    overflow: hidden;
    text-align: right;
    user-select: none;
    color: var(--glass-text-faint);
    border-right: 1px solid var(--glass-border);
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 13px;
    line-height: 1.55;
    font-variant-numeric: tabular-nums;
  }
  textarea:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
</style>
