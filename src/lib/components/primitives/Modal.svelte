<script lang="ts">
  let {
    open = $bindable(false),
    title = '',
    children,
  }: {
    open?: boolean;
    title?: string;
    children?: import('svelte').Snippet;
  } = $props();

  function close() {
    open = false;
  }
</script>

{#if open}
  <div class="backdrop" role="presentation" onclick={close}></div>
  <div class="modal glass-elevated spring" role="dialog" aria-modal="true" aria-label={title || 'Dialog'}>
    {#if title}
      <header class="head">
        <h2>{title}</h2>
        <button type="button" class="close" onclick={close} aria-label="Close">×</button>
      </header>
    {/if}
    <div class="body">
      {#if children}{@render children()}{/if}
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(10, 14, 36, 0.6);
    z-index: 100;
  }
  .modal {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: min(520px, 92vw);
    max-height: 86vh;
    overflow: auto;
    padding: 20px 24px;
    z-index: 101;
    color: var(--glass-text);
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }
  h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }
  .close {
    width: 28px;
    height: 28px;
    padding: 0;
    background: transparent;
    border: 1px solid var(--glass-border);
    border-radius: 50%;
    color: var(--glass-text);
    font-size: 16px;
    line-height: 1;
    cursor: pointer;
  }
  .close:hover {
    background: rgba(255, 255, 255, 0.08);
  }
  .close:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 1px;
  }
  .body {
    color: var(--glass-text-dim);
    font-size: 14px;
    line-height: 1.5;
  }
</style>
