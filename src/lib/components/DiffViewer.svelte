<script lang="ts">
  import { onMount } from 'svelte';
  import { t, locale } from '$lib/i18n';

  type DiffSide = 'base' | 'ours' | 'theirs';
  type DiffTone = 'base' | 'same' | 'ours-change' | 'theirs-change' | 'conflict' | 'marker' | 'empty';
  type DiffOperation =
    | { kind: 'match'; baseIndex: number; sideIndex: number }
    | { kind: 'insert'; gap: number; sideIndex: number }
    | { kind: 'delete'; baseIndex: number };

  interface DiffLine {
    number: number;
    text: string;
    tone: DiffTone;
  }

  interface DiffRow {
    base: DiffLine;
    ours: DiffLine;
    theirs: DiffLine;
  }

  interface SideAlignment {
    matches: Map<number, number>;
    insertions: Map<number, number[]>;
  }

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

  function splitLines(value: string): string[] {
    return value.length === 0 ? [] : value.split(/\r?\n/);
  }

  function isConflictMarker(value: string): boolean {
    return value.startsWith('<<<<<<<') || value === '=======' || value.startsWith('>>>>>>>');
  }

  function lcsTable(baseLines: string[], sideLines: string[]): number[][] {
    const table = Array.from({ length: baseLines.length + 1 }, () =>
      Array<number>(sideLines.length + 1).fill(0),
    );
    for (let baseIndex = baseLines.length - 1; baseIndex >= 0; baseIndex -= 1) {
      for (let sideIndex = sideLines.length - 1; sideIndex >= 0; sideIndex -= 1) {
        table[baseIndex][sideIndex] =
          baseLines[baseIndex] === sideLines[sideIndex]
            ? table[baseIndex + 1][sideIndex + 1] + 1
            : Math.max(table[baseIndex + 1][sideIndex], table[baseIndex][sideIndex + 1]);
      }
    }
    return table;
  }

  function alignSide(baseLines: string[], sideLines: string[]): SideAlignment {
    const table = lcsTable(baseLines, sideLines);
    const operations: DiffOperation[] = [];
    let baseIndex = 0;
    let sideIndex = 0;

    while (baseIndex < baseLines.length || sideIndex < sideLines.length) {
      if (
        baseIndex < baseLines.length &&
        sideIndex < sideLines.length &&
        baseLines[baseIndex] === sideLines[sideIndex]
      ) {
        operations.push({ kind: 'match', baseIndex, sideIndex });
        baseIndex += 1;
        sideIndex += 1;
      } else if (
        sideIndex < sideLines.length &&
        (baseIndex === baseLines.length || table[baseIndex][sideIndex + 1] >= table[baseIndex + 1][sideIndex])
      ) {
        operations.push({ kind: 'insert', gap: baseIndex, sideIndex });
        sideIndex += 1;
      } else {
        operations.push({ kind: 'delete', baseIndex });
        baseIndex += 1;
      }
    }

    const matches = new Map<number, number>();
    const insertions = new Map<number, number[]>();
    let operationIndex = 0;
    while (operationIndex < operations.length) {
      const operation = operations[operationIndex];
      if (operation.kind === 'match') {
        matches.set(operation.baseIndex, operation.sideIndex);
        operationIndex += 1;
        continue;
      }

      const deletes: Extract<DiffOperation, { kind: 'delete' }>[] = [];
      const inserts: Extract<DiffOperation, { kind: 'insert' }>[] = [];
      while (operationIndex < operations.length && operations[operationIndex].kind !== 'match') {
        const current = operations[operationIndex];
        if (current.kind === 'delete') deletes.push(current);
        else if (current.kind === 'insert') inserts.push(current);
        operationIndex += 1;
      }

      const pairedCount = Math.min(deletes.length, inserts.length);
      for (let index = 0; index < pairedCount; index += 1) {
        matches.set(deletes[index].baseIndex, inserts[index].sideIndex);
      }
      for (const insert of inserts.slice(pairedCount)) {
        const lines = insertions.get(insert.gap) ?? [];
        lines.push(insert.sideIndex);
        insertions.set(insert.gap, lines);
      }
    }

    return { matches, insertions };
  }

  function toneFor(
    side: DiffSide,
    text: string,
    exists: boolean,
    baseText: string | undefined,
    oursText: string | undefined,
    theirsText: string | undefined,
  ): DiffTone {
    if (!exists) return 'empty';
    if (isConflictMarker(text)) return 'marker';

    const hasConflict =
      oursText !== undefined &&
      theirsText !== undefined &&
      oursText !== theirsText &&
      (baseText === undefined || (oursText !== baseText && theirsText !== baseText));
    if (hasConflict) return 'conflict';
    if (side === 'base') return 'base';
    return text !== baseText ? (side === 'ours' ? 'ours-change' : 'theirs-change') : 'same';
  }

  function makeLine(
    side: DiffSide,
    index: number | undefined,
    lines: string[],
    baseText: string | undefined,
    oursText: string | undefined,
    theirsText: string | undefined,
  ): DiffLine {
    const exists = index !== undefined;
    const text = exists ? lines[index] : '';
    return {
      number: exists ? index + 1 : 0,
      text,
      tone: toneFor(side, text, exists, baseText, oursText, theirsText),
    };
  }

  const rows = $derived.by(() => {
    const baseLines = splitLines(base);
    const oursLines = splitLines(ours);
    const theirsLines = splitLines(theirs);
    const oursAlignment = alignSide(baseLines, oursLines);
    const theirsAlignment = alignSide(baseLines, theirsLines);
    const alignedRows: DiffRow[] = [];

    const addRow = (baseIndex: number | undefined, oursIndex: number | undefined, theirsIndex: number | undefined) => {
      const baseText = baseIndex === undefined ? undefined : baseLines[baseIndex];
      const oursText = oursIndex === undefined ? undefined : oursLines[oursIndex];
      const theirsText = theirsIndex === undefined ? undefined : theirsLines[theirsIndex];
      alignedRows.push({
        base: makeLine('base', baseIndex, baseLines, baseText, oursText, theirsText),
        ours: makeLine('ours', oursIndex, oursLines, baseText, oursText, theirsText),
        theirs: makeLine('theirs', theirsIndex, theirsLines, baseText, oursText, theirsText),
      });
    };

    for (let gap = 0; gap <= baseLines.length; gap += 1) {
      const oursInserted = oursAlignment.insertions.get(gap) ?? [];
      const theirsInserted = theirsAlignment.insertions.get(gap) ?? [];
      const insertedCount = Math.max(oursInserted.length, theirsInserted.length);
      for (let index = 0; index < insertedCount; index += 1) {
        addRow(undefined, oursInserted[index], theirsInserted[index]);
      }

      if (gap < baseLines.length) {
        addRow(gap, oursAlignment.matches.get(gap), theirsAlignment.matches.get(gap));
      }
    }

    return alignedRows;
  });

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
          {#each rows as row}
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
