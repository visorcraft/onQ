<script lang="ts">
  let {
    label,
    active = false,
    removable = false,
    onRemove,
  }: {
    label: string;
    active?: boolean;
    removable?: boolean;
    onRemove?: () => void;
  } = $props();
</script>

<span class="chip" class:active aria-pressed={active}>
  <span class="text">{label}</span>
  {#if removable}
    <button type="button" class="x" aria-label={`Remove ${label}`} onclick={onRemove}>×</button>
  {/if}
</span>

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    color: var(--glass-text);
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--glass-border);
    border-radius: 999px;
    font-size: 12px;
    line-height: 1.4;
    transition: background var(--motion-duration) ease, border-color var(--motion-duration) ease;
  }
  .chip.active {
    background: rgba(91, 143, 255, 0.22);
    border-color: rgba(184, 210, 255, 0.48);
    color: #f8fbff;
  }
  .text {
    user-select: none;
  }
  .x {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    padding: 0;
    background: transparent;
    border: 0;
    border-radius: 50%;
    color: inherit;
    font-size: 13px;
    line-height: 1;
    cursor: pointer;
  }
  .x:hover {
    background: rgba(255, 255, 255, 0.12);
  }
  .x:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 1px;
  }
</style>
