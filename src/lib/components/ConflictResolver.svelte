<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import DiffViewer from './DiffViewer.svelte';
  import {
    parseConflictText,
    renderConflict,
    type ConflictChoice,
    type ConflictHunk,
    type ConflictSegment,
  } from '$lib/api/sync';

  type HunkResolution = ConflictChoice | 'manual';

  // The callback parameter name is intentionally omitted from lint coverage;
  // this alias only describes the public component contract.
  // eslint-disable-next-line no-unused-vars
  type ResolvedCallback = (resolved: string) => void;

  let {
    conflictText = '',
    base,
    onResolved,
    onResolve,
  }: {
    conflictText?: string;
    base?: string;
    onResolved?: ResolvedCallback;
    onResolve?: ResolvedCallback;
  } = $props();

  let choices = $state<Record<number, HunkResolution>>({});
  let manualHunkText = $state<Record<number, string>>({});
  let manualText = $state(untrack(() => conflictText));
  let editingHunkId = $state<number | null>(null);
  let manualError = $state('');
  let dialog: HTMLDivElement;

  onMount(() => dialog?.focus());

  const segments = $derived.by(() => parseConflictText(conflictText));
  const hunks = $derived(
    segments.filter((segment): segment is ConflictHunk => segment.kind === 'hunk'),
  );
  const hasHunks = $derived(hunks.length > 0);
  const resolvedCount = $derived(
    hunks.filter((hunk) => choices[hunk.id] !== undefined).length,
  );
  const allHunksResolved = $derived(hasHunks && resolvedCount === hunks.length);

  const baseVersion = $derived(base ?? '');
  const oursVersion = $derived.by(() => renderChoice('ours'));
  const theirsVersion = $derived.by(() => renderChoice('theirs'));
  const resolvedPreview = $derived.by(() => renderResolved());
  const statusMessage = $derived(
    hasHunks
      ? `${resolvedCount} of ${hunks.length} conflict ${hunks.length === 1 ? 'hunk' : 'hunks'} resolved`
      : 'No conflict markers found',
  );

  // Keep the manual editor in sync when a new conflict arrives while the
  // resolver remains mounted.
  $effect(() => {
    manualText = conflictText;
    choices = {};
    manualHunkText = {};
    editingHunkId = null;
    manualError = '';
  });

  function renderChoice(choice: ConflictChoice): string {
    const map = new Map<number, ConflictChoice>();
    for (const hunk of hunks) map.set(hunk.id, choice);
    return renderConflict(segments, map);
  }

  function renderResolved(): string {
    return segments
      .map((segment: ConflictSegment) => {
        if (segment.kind === 'context') return segment.text;
        const resolution = choices[segment.id];
        if (resolution === 'manual') return manualHunkText[segment.id] ?? '';
        if (resolution === 'ours') return segment.ours;
        if (resolution === 'theirs') return segment.theirs;
        if (resolution === 'both') {
          return [segment.ours, segment.theirs].filter((part) => part.length > 0).join('\n');
        }
        return `<<<<<<< ours\n${segment.ours}\n=======\n${segment.theirs}\n>>>>>>> theirs`;
      })
      .join('\n');
  }

  function notifyResolved(result: string): void {
    (onResolved ?? onResolve)?.(result);
  }

  function resolveAll(choice: ConflictChoice): void {
    const next: Record<number, HunkResolution> = {};
    for (const hunk of hunks) next[hunk.id] = choice;
    choices = next;
    editingHunkId = null;
    manualError = '';
    notifyResolved(renderChoice(choice));
  }

  function resolveHunk(id: number, choice: ConflictChoice): void {
    choices = { ...choices, [id]: choice };
    editingHunkId = null;
    manualError = '';
    if (hunks.every((hunk) => choices[hunk.id] !== undefined)) {
      notifyResolved(renderResolved());
    }
  }

  function startManualEdit(): void {
    manualText = resolvedPreview;
    manualError = '';
  }

  function applyManualEdit(): void {
    const stillConflicted = parseConflictText(manualText).some((segment) => segment.kind === 'hunk');
    if (stillConflicted) {
      manualError = 'Resolve every conflict block before applying the manual resolution.';
      return;
    }
    manualError = '';
    notifyResolved(manualText);
  }

  function startHunkEdit(hunk: ConflictHunk): void {
    editingHunkId = hunk.id;
    manualHunkText = { ...manualHunkText, [hunk.id]: hunk.ours };
    manualError = '';
  }

  function applyHunkEdit(hunk: ConflictHunk): void {
    const value = manualHunkText[hunk.id] ?? '';
    choices = { ...choices, [hunk.id]: 'manual' };
    manualHunkText = { ...manualHunkText, [hunk.id]: value };
    editingHunkId = null;
    manualError = '';
    if (hunks.every((item) => choices[item.id] !== undefined)) {
      notifyResolved(renderResolved());
    }
  }
</script>

<div
  class="resolver"
  role="dialog"
  aria-modal="true"
  aria-label="Resolve sync conflict"
  tabindex="-1"
  bind:this={dialog}
>
  <div class="resolver-header">
    <div>
      <p class="eyebrow">Merge requires your decision</p>
      <h1>Resolve conflict</h1>
      <p class="description">Choose how to combine the current vault with the incoming edit.</p>
    </div>
    <p class="status" aria-live="polite">{statusMessage}</p>
  </div>

  <DiffViewer base={baseVersion} ours={oursVersion} theirs={theirsVersion} embedded />

  <section class="resolution-panel" aria-label="Conflict resolution actions">
    <div class="panel-heading">
      <div>
        <h2>Choose a resolution</h2>
        <p>Apply one choice to every hunk, or resolve them individually below.</p>
      </div>
      {#if allHunksResolved}
        <span class="resolved-badge">Ready to apply</span>
      {/if}
    </div>

    <div class="bulk-actions">
      <button type="button" class="primary" onclick={() => resolveAll('ours')}>Accept ours</button>
      <button type="button" class="secondary" onclick={() => resolveAll('theirs')}>Accept theirs</button>
      <button type="button" class="secondary" onclick={() => resolveAll('both')}>Keep both</button>
      <button type="button" class="quiet" onclick={startManualEdit}>Edit manually</button>
    </div>

    <div class="hunks" aria-label="Per-hunk resolutions">
      {#if hasHunks}
        {#each hunks as hunk, index (hunk.id)}
          {@const selected = choices[hunk.id]}
          <article class="hunk" class:selected={selected !== undefined}>
            <div class="hunk-header">
              <div>
                <h3>Conflict {index + 1}</h3>
                <span>{selected ? `Using ${selected}` : 'Needs a choice'}</span>
              </div>
              <div class="hunk-actions">
                <button
                  type="button"
                  class:active={selected === 'ours'}
                  onclick={() => resolveHunk(hunk.id, 'ours')}
                >Take ours</button>
                <button
                  type="button"
                  class:active={selected === 'theirs'}
                  onclick={() => resolveHunk(hunk.id, 'theirs')}
                >Take theirs</button>
                <button
                  type="button"
                  class:active={selected === 'both'}
                  onclick={() => resolveHunk(hunk.id, 'both')}
                >Keep both</button>
                <button type="button" class:active={selected === 'manual'} onclick={() => startHunkEdit(hunk)}>
                  Edit
                </button>
              </div>
            </div>

            {#if editingHunkId === hunk.id}
              <div class="hunk-editor">
                <label for={`hunk-edit-${hunk.id}`}>Manual text for conflict {index + 1}</label>
                <textarea
                  id={`hunk-edit-${hunk.id}`}
                  value={manualHunkText[hunk.id] ?? ''}
                  oninput={(event) =>
                    (manualHunkText = {
                      ...manualHunkText,
                      [hunk.id]: (event.currentTarget as HTMLTextAreaElement).value,
                    })}
                ></textarea>
                <button type="button" class="primary small" onclick={() => applyHunkEdit(hunk)}>Apply edit</button>
              </div>
            {:else}
              <div class="hunk-preview">
                <div>
                  <span class="preview-label ours-label">Ours</span>
                  <pre>{hunk.ours || ' '}</pre>
                </div>
                <div>
                  <span class="preview-label theirs-label">Theirs</span>
                  <pre>{hunk.theirs || ' '}</pre>
                </div>
              </div>
            {/if}
          </article>
        {/each}
      {:else}
        <p class="empty">This text is already clean. Applying it will keep the current content unchanged.</p>
      {/if}
    </div>
  </section>

  <section class="manual-panel" aria-label="Manual resolution editor">
    <div class="panel-heading">
      <div>
        <h2>Manual resolution</h2>
        <p>Use this editor when the automatic choices do not fit.</p>
      </div>
    </div>
    <textarea
      aria-label="Resolved conflict text"
      bind:value={manualText}
      placeholder="Edit the merged prompt here…"
    ></textarea>
    {#if manualError}
      <p class="error" role="alert">{manualError}</p>
    {/if}
    <div class="manual-actions">
      <button type="button" class="primary" onclick={applyManualEdit}>Apply manual resolution</button>
    </div>
  </section>
</div>

<style>
  .resolver {
    width: min(1220px, 97vw);
    max-height: 96vh;
    overflow: auto;
    padding: 24px;
    color: var(--glass-text);
    background: rgba(19, 27, 52, 0.88);
    border: 1px solid var(--glass-border-strong);
    border-radius: var(--glass-radius-lg);
    box-shadow: 0 28px 90px rgba(4, 8, 24, 0.48), inset 0 1px rgba(255, 255, 255, 0.08);
    backdrop-filter: blur(var(--glass-blur-lg));
    -webkit-backdrop-filter: blur(var(--glass-blur-lg));
  }

  .resolver:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 4px;
  }

  .resolver-header,
  .panel-heading,
  .hunk-header,
  .manual-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .resolver-header {
    align-items: end;
    margin-bottom: 16px;
  }

  .eyebrow {
    margin: 0 0 5px;
    color: #f5b141;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  h1,
  h2,
  h3,
  p {
    margin: 0;
  }

  h1 {
    font-size: clamp(22px, 2.3vw, 30px);
    letter-spacing: -0.025em;
  }

  h2 {
    font-size: 15px;
  }

  h3 {
    font-size: 13px;
  }

  .description,
  .panel-heading p,
  .hunk-header span,
  .status,
  .empty {
    color: var(--glass-text-dim);
    font-size: 12px;
    line-height: 1.5;
  }

  .description {
    margin-top: 6px;
  }

  .status {
    flex: 0 0 auto;
    padding: 7px 10px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--glass-border);
    border-radius: 999px;
    font-variant-numeric: tabular-nums;
  }

  :global(.resolver .viewer) {
    width: 100%;
    max-height: none;
    padding: 0;
    overflow: visible;
    background: transparent;
    border: 0;
    box-shadow: none;
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
  }

  :global(.resolver .viewer-header) {
    display: none;
  }

  .resolution-panel,
  .manual-panel {
    margin-top: 14px;
    padding: 17px;
    background: rgba(255, 255, 255, 0.045);
    border: 1px solid var(--glass-border);
    border-radius: var(--glass-radius);
  }

  .panel-heading {
    align-items: start;
  }

  .panel-heading p {
    margin-top: 4px;
  }

  .resolved-badge {
    flex: 0 0 auto;
    padding: 5px 9px;
    color: #b9f3d0;
    background: rgba(53, 184, 111, 0.14);
    border: 1px solid rgba(95, 224, 148, 0.3);
    border-radius: 999px;
    font-size: 11px;
  }

  .bulk-actions,
  .hunk-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .bulk-actions {
    margin-top: 16px;
  }

  button {
    padding: 8px 12px;
    color: var(--glass-text);
    background: rgba(255, 255, 255, 0.065);
    border: 1px solid var(--glass-border);
    border-radius: 9px;
    cursor: pointer;
    font: inherit;
    font-size: 12px;
    transition: background var(--motion-duration) ease, border-color var(--motion-duration) ease,
      transform var(--motion-duration) var(--motion-spring);
  }

  button:hover {
    background: rgba(255, 255, 255, 0.12);
    border-color: var(--glass-border-strong);
  }

  button:active {
    transform: translateY(1px);
  }

  button:focus-visible,
  textarea:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }

  button.primary {
    color: #f8fbff;
    background: var(--glass-accent);
    border-color: transparent;
  }

  button.primary:hover {
    background: #78a3ff;
  }

  button.small {
    padding: 7px 10px;
  }

  button.quiet {
    color: var(--glass-text-dim);
    background: transparent;
  }

  button.active {
    background: rgba(91, 143, 255, 0.22);
    border-color: rgba(184, 210, 255, 0.48);
  }

  .hunks {
    display: grid;
    gap: 10px;
    margin-top: 16px;
  }

  .hunk {
    overflow: hidden;
    background: rgba(7, 13, 34, 0.25);
    border: 1px solid var(--glass-border);
    border-radius: 11px;
  }

  .hunk.selected {
    border-color: rgba(91, 143, 255, 0.4);
  }

  .hunk-header {
    align-items: start;
    padding: 11px 12px;
    border-bottom: 1px solid var(--glass-border);
  }

  .hunk-actions {
    justify-content: end;
  }

  .hunk-actions button {
    padding: 6px 8px;
    font-size: 11px;
  }

  .hunk-preview {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
    padding: 11px 12px 13px;
  }

  .preview-label {
    display: inline-block;
    margin-bottom: 6px;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .ours-label {
    color: var(--glass-accent);
  }

  .theirs-label {
    color: var(--glass-periwinkle);
  }

  pre {
    min-height: 38px;
    margin: 0;
    padding: 9px;
    overflow: auto;
    color: var(--glass-text-dim);
    background: rgba(0, 0, 0, 0.18);
    border: 1px solid var(--glass-border);
    border-radius: 7px;
    font-family: 'JetBrains Mono', 'SFMono-Regular', Consolas, monospace;
    font-size: 11px;
    line-height: 1.55;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .hunk-editor,
  .manual-panel {
    display: grid;
    gap: 10px;
  }

  .hunk-editor {
    padding: 11px 12px 13px;
  }

  label {
    color: var(--glass-text-dim);
    font-size: 11px;
  }

  textarea {
    width: 100%;
    min-height: 95px;
    box-sizing: border-box;
    padding: 10px 11px;
    color: var(--glass-text);
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--glass-border);
    border-radius: 9px;
    font: 12px/1.6 'JetBrains Mono', 'SFMono-Regular', Consolas, monospace;
    resize: vertical;
  }

  .manual-panel textarea {
    min-height: 130px;
  }

  .error {
    color: #ffb5ae;
    font-size: 12px;
  }

  .manual-actions {
    justify-content: flex-end;
  }

  @media (max-width: 720px) {
    .resolver {
      width: calc(100vw - 12px);
      padding: 15px;
    }

    .resolver-header,
    .panel-heading,
    .hunk-header {
      align-items: start;
      flex-direction: column;
    }

    .status {
      align-self: start;
    }

    .hunk-actions {
      justify-content: start;
    }

    .hunk-preview {
      grid-template-columns: 1fr;
    }
  }

  @media (prefers-contrast: more) {
    .resolver {
      background: var(--glass-surface);
      backdrop-filter: none;
      -webkit-backdrop-filter: none;
    }
  }
</style>
