<script lang="ts">
  import { visualToDsl, type VisualPredicate, type VisualQuery } from '$lib/api/smartFolderVisual';
  import { t, locale } from '$lib/i18n';

  let {
    predicates = $bindable<VisualPredicate[]>([]),
    onDslChange,
  }: {
    predicates?: VisualPredicate[];
    onDslChange?: (dsl: string) => void; // eslint-disable-line no-unused-vars
  } = $props();

  let field = $state('tag');
  let op = $state('is');
  let value = $state('');
  let error = $state<string | null>(null);

  const fieldOps: Record<string, string[]> = {
    tag: ['is', 'not'],
    folder: ['is'],
    favorite: ['is'],
    locked: ['is'],
    text: ['contains'],
  };

  $effect(() => {
    // Reset op when field changes to first valid.
    const ops = fieldOps[field] ?? ['is'];
    if (!ops.includes(op)) op = ops[0];
  });

  async function emitDsl(next: VisualPredicate[]) {
    error = null;
    try {
      const visual: VisualQuery = { predicates: next };
      const dsl = await visualToDsl(visual);
      onDslChange?.(dsl);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function addChip() {
    let v = value.trim();
    if (field === 'favorite' || field === 'locked') {
      v = v === 'false' || v === '0' ? 'false' : 'true';
    } else if (!v) {
      error = t('smart.valueRequired');
      return;
    }
    const next = [...predicates, { field, op, value: v }];
    predicates = next;
    value = '';
    await emitDsl(next);
  }

  async function removeChip(index: number) {
    const next = predicates.filter((_, i) => i !== index);
    predicates = next;
    await emitDsl(next);
  }
</script>

<div class="visual-builder" aria-label={t('smart.builderAria', undefined, $locale)}>
  <div class="chips">
    {#each predicates as p, i (i)}
      <button
        type="button"
        class="chip"
        onclick={() => void removeChip(i)}
        title={t('common.remove', undefined, $locale)}
      >
        {p.field}:{p.op === 'not' ? '!' : ''}{p.value}
        <span aria-hidden="true">×</span>
      </button>
    {/each}
    {#if predicates.length === 0}
      <span class="empty">{t('smart.noFilters', undefined, $locale)}</span>
    {/if}
  </div>
  <div class="add-row">
    <select bind:value={field} aria-label={t('smart.field', undefined, $locale)}>
      <option value="tag">tag</option>
      <option value="folder">folder</option>
      <option value="favorite">favorite</option>
      <option value="locked">locked</option>
      <option value="text">text</option>
    </select>
    <select bind:value={op} aria-label={t('smart.operator', undefined, $locale)}>
      {#each fieldOps[field] ?? ['is'] as o (o)}
        <option value={o}>{o}</option>
      {/each}
    </select>
    {#if field === 'favorite' || field === 'locked'}
      <select bind:value={value} aria-label={t('smart.value', undefined, $locale)}>
        <option value="true">true</option>
        <option value="false">false</option>
      </select>
    {:else}
      <input
        type="text"
        bind:value
        placeholder={t('smart.valuePlaceholder', undefined, $locale)}
        aria-label={t('smart.value', undefined, $locale)}
      />
    {/if}
    <button type="button" class="control-btn sm" onclick={() => void addChip()}
      >{t('common.add', undefined, $locale)}</button
    >
  </div>
  {#if error}
    <p class="err" role="alert">{error}</p>
  {/if}
</div>

<style>
  .visual-builder {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    margin: 0.35rem 0;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    min-height: 1.6rem;
  }
  .chip {
    border: 1px solid var(--glass-border, #333);
    background: color-mix(in oklab, var(--glass-accent, #7aa2ff) 18%, transparent);
    color: var(--glass-text, #eee);
    border-radius: 999px;
    padding: 0.15rem 0.55rem;
    font-size: 0.75rem;
    cursor: pointer;
  }
  .empty {
    font-size: 0.75rem;
    opacity: 0.6;
  }
  .add-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    align-items: center;
  }
  .add-row select,
  .add-row input {
    font: inherit;
    font-size: 0.8rem;
    padding: 0.25rem 0.4rem;
    border-radius: 6px;
    border: 1px solid var(--glass-border, #333);
    background: var(--glass-control-bg, #1a1a22);
    color: inherit;
    max-width: 8rem;
  }
  .err {
    color: #f87171;
    font-size: 0.75rem;
    margin: 0;
  }
</style>
