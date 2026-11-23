<script lang="ts">
  import { onMount } from 'svelte';
  import { t, locale } from '$lib/i18n';
  import { buildDiffRows, splitLines, type DiffSide } from './diff-utils';

  let {
    base = '',
    ours = '',
    theirs = '',
    embedded = false,
  }: {
    base?: string;
    ours?: string;
    theirs?: string;
    embedded?: boolean;
  } = $props();

  let dialog: HTMLDivElement;

  onMount(() => {
    if (!embedded) dialog?.focus();
  });

  const rows = $derived(buildDiffRows(base, ours, theirs));

  const baseLineCount = $derived(splitLines(base).length);
  const oursLineCount = $derived(splitLines(ours).length);
  const theirsLineCount = $derived(splitLines(theirs).length);
</script>

<div
  class="viewer"
  role={embedded ? 'region' : 'dialog'}
  aria-modal={embedded ? undefined : 'true'}
  aria-label={t('diff.aria', undefined, $locale)}
  tabindex="-1"
  bind:this={dialog}
>
  <header class="viewer-header">
    <div>
      <p class="eyebrow">{t('diff.eyebrow', undefined, $locale)}</p>
      <h2>{t('diff.title', undefined, $locale)}</h2>
    </div>
    <p class="hint">{t('diff.hint', undefined, $locale)}</p>
  </header>

  <div class="columns" aria-label={t('diff.columnsAria', undefined, $locale)}>
    {#each [
      { key: 'base', label: t('diff.base', undefined, $locale), detail: t('diff.baseDetail', undefined, $locale), count: baseLineCount },
      { key: 'ours', label: t('conflict.ours', undefined, $locale), detail: t('diff.oursDetail', undefined, $locale), count: oursLineCount },
      { key: 'theirs', label: t('conflict.theirs', undefined, $locale), detail: t('diff.theirsDetail', undefined, $locale), count: theirsLineCount },
    ] as column (column.key)}
      <section class:ours-column={column.key === 'ours'} class:theirs-column={column.key === 'theirs'} class="column" aria-label={`${column.label} version`}>
        <header class="column-header">
          <div>
            <h3>{column.label}</h3>
            <span>{column.detail}</span>
          </div>
          <span class="line-count">{column.count} lines</span>
        </header>
        <div class="code-panel" role="region" aria-label={`${column.label} content`}>
          {#each rows as row, index (index)}
            {@const line = row[column.key as DiffSide]}
            <div
              class="code-line"
              class:line-changed={line.tone === 'ours-change' || line.tone === 'theirs-change'}
              class:line-conflict={line.tone === 'conflict'}
              class:line-marker={line.tone === 'marker'}
              class:line-empty={line.tone === 'empty'}
              aria-label={`${column.label} line ${line.number}`}
            >
              <span class="line-number" aria-hidden="true">{line.number || '·'}</span>
              <code>{line.text || ' '}</code>
            </div>
          {/each}
        </div>
      </section>
    {/each}
  </div>
</div>

<style>
  .viewer {
    width: min(1180px, 96vw);
    max-height: min(88vh, 900px);
    padding: 22px;
    overflow: hidden;
    color: var(--glass-text);
    background: rgba(25, 33, 58, 0.76);
    border: 1px solid var(--glass-border-strong);
    border-radius: var(--glass-radius-lg);
    box-shadow: 0 24px 72px rgba(4, 8, 24, 0.35), inset 0 1px rgba(255, 255, 255, 0.08);
    backdrop-filter: blur(var(--glass-blur-lg));
    -webkit-backdrop-filter: blur(var(--glass-blur-lg));
  }

  .viewer:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 4px;
  }

  .viewer-header {
    display: flex;
    align-items: end;
    justify-content: space-between;
    gap: 20px;
    margin-bottom: 18px;
  }

  .eyebrow {
    margin: 0 0 5px;
    color: var(--glass-accent);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  h2,
  h3,
  p {
    margin: 0;
  }

  h2 {
    font-size: clamp(20px, 2vw, 26px);
    letter-spacing: -0.02em;
  }

  .hint {
    max-width: 260px;
    color: var(--glass-text-dim);
    font-size: 12px;
    line-height: 1.5;
    text-align: right;
  }

  .columns {
    display: grid;
    grid-template-columns: repeat(3, minmax(240px, 1fr));
    gap: 10px;
    overflow: auto;
    padding: 1px;
  }

  .column {
    min-width: 0;
    overflow: hidden;
    background: rgba(6, 12, 32, 0.31);
    border: 1px solid var(--glass-border);
    border-radius: var(--glass-radius);
  }

  .ours-column {
    border-color: rgba(91, 143, 255, 0.34);
  }

  .theirs-column {
    border-color: rgba(184, 210, 255, 0.32);
  }

  .column-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    min-height: 58px;
    padding: 11px 13px;
    background: rgba(255, 255, 255, 0.055);
    border-bottom: 1px solid var(--glass-border);
  }

  h3 {
    font-size: 14px;
    font-weight: 650;
  }

  .column-header span {
    display: block;
    margin-top: 3px;
    color: var(--glass-text-faint);
    font-size: 11px;
  }

  .column-header .line-count {
    flex: 0 0 auto;
    margin-top: 0;
    color: var(--glass-text-dim);
    font-variant-numeric: tabular-nums;
  }

  .code-panel {
    max-height: min(58vh, 620px);
    overflow: auto;
    padding: 8px 0;
    font-family: 'JetBrains Mono', 'SFMono-Regular', Consolas, monospace;
    font-size: 12px;
    line-height: 1.65;
  }

  .code-line {
    display: grid;
    grid-template-columns: 38px minmax(0, 1fr);
    min-height: 20px;
    border-left: 2px solid transparent;
  }

  .code-line code {
    min-width: 0;
    padding: 0 10px;
    overflow-wrap: anywhere;
    white-space: pre-wrap;
  }

  .line-number {
    padding-right: 8px;
    color: rgba(168, 184, 216, 0.48);
    text-align: right;
    user-select: none;
  }

  .line-changed {
    background: rgba(91, 143, 255, 0.12);
    border-left-color: rgba(91, 143, 255, 0.75);
  }

  .line-conflict {
    background: rgba(245, 177, 65, 0.18);
    border-left-color: #f5b141;
  }

  .line-marker {
    background: rgba(245, 177, 65, 0.1);
    color: #ffd58a;
    border-left-color: rgba(245, 177, 65, 0.65);
  }

  .line-empty {
    opacity: 0.36;
  }

  @media (max-width: 700px) {
    .viewer {
      width: calc(100vw - 20px);
      padding: 15px;
    }

    .viewer-header {
      align-items: start;
      flex-direction: column;
      gap: 8px;
    }

    .hint {
      max-width: none;
      text-align: left;
    }

    .columns {
      grid-template-columns: repeat(3, minmax(260px, 1fr));
    }
  }

  @media (prefers-contrast: more) {
    .viewer {
      background: var(--glass-surface);
      backdrop-filter: none;
      -webkit-backdrop-filter: none;
    }

    .line-conflict {
      outline: 1px solid #f5b141;
      outline-offset: -1px;
    }
  }
</style>
