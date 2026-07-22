<script lang="ts">
  /**
   * Compact pagination control used by every Library list view. Caller owns
   * the source list and the current page index — the component is stateless
   * so switching selection / filter resets are trivial at the call site.
   *
   * Visual goal: feels native to the onQ glass shell (rounded pill on the
   * right of the list, glass border, prev/next that hug the indicator).
   */
  let {
    page,
    pageCount,
    onPage,
    ariaLabel = 'Pagination',
  }: {
    /** 1-based current page index. */
    page: number;
    /** Total page count (>=1). */
    pageCount: number;
    // The parameter name is for documentation only — the call site is
    // `onPage(next)`, never the parameter name. eslint can't see that
    // through the function-type position, so silence it locally.
    // eslint-disable-next-line no-unused-vars
    onPage: (pageIndex: number) => void;
    ariaLabel?: string;
  } = $props();

  const canPrev = $derived(page > 1);
  const canNext = $derived(page < pageCount);

  function goto(next: number) {
    if (next < 1 || next > pageCount || next === page) return;
    onPage(next);
  }
</script>

{#if pageCount > 1}
  <nav class="pager" aria-label={ariaLabel}>
    <button
      type="button"
      class="pager-step"
      aria-label="Previous page"
      disabled={!canPrev}
      onclick={() => goto(page - 1)}
    >
      <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true">
        <path
          d="M10 2.5 4.5 8 10 13.5"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
    <span class="pager-indicator" aria-current="page">
      <span class="pager-current">{page}</span>
      <span class="pager-divider">/</span>
      <span class="pager-total">{pageCount}</span>
    </span>
    <button
      type="button"
      class="pager-step"
      aria-label="Next page"
      disabled={!canNext}
      onclick={() => goto(page + 1)}
    >
      <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true">
        <path
          d="M6 2.5 11.5 8 6 13.5"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
  </nav>
{/if}

<style>
  .pager {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 6px;
    border-radius: 999px;
    border: 1px solid var(--glass-border);
    background: var(--glass-control-bg);
    box-shadow: var(--glass-inset-highlight);
  }
  .pager-step {
    appearance: none;
    border: 0;
    background: transparent;
    color: var(--glass-text);
    width: 28px;
    height: 28px;
    border-radius: 999px;
    cursor: pointer;
    display: grid;
    place-items: center;
    transition:
      background var(--motion-duration) ease,
      color var(--motion-duration) ease,
      transform var(--motion-duration) var(--motion-spring);
  }
  .pager-step:hover:not(:disabled) {
    background: var(--glass-hover-strong);
  }
  .pager-step:active:not(:disabled) {
    transform: scale(0.95);
  }
  .pager-step:disabled {
    color: var(--glass-text-faint);
    cursor: not-allowed;
    opacity: 0.55;
  }
  .pager-step:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .pager-indicator {
    display: inline-flex;
    align-items: baseline;
    gap: 4px;
    padding: 0 8px;
    font-variant-numeric: tabular-nums;
    color: var(--glass-text);
    font-size: 13px;
    font-weight: 600;
    line-height: 1;
    min-width: 64px;
    justify-content: center;
  }
  .pager-current {
    color: var(--glass-text);
  }
  .pager-divider {
    color: var(--glass-text-faint);
    font-weight: 500;
  }
  .pager-total {
    color: var(--glass-text-dim);
    font-weight: 500;
  }
</style>