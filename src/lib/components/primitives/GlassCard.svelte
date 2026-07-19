<script lang="ts">
  let {
    title = '',
    elevated = false,
    children,
    footer,
  }: {
    title?: string;
    elevated?: boolean;
    children?: import('svelte').Snippet;
    footer?: import('svelte').Snippet;
  } = $props();
</script>

<section class="card {elevated ? 'glass-elevated' : 'glass'} spring">
  {#if title}
    <header class="head"><h3>{title}</h3></header>
  {/if}
  <div class="body">
    {#if children}{@render children()}{/if}
  </div>
  {#if footer}
    <footer class="foot">{@render footer()}</footer>
  {/if}
</section>

<style>
  .card {
    padding: 18px 20px;
    color: var(--glass-text);
    box-shadow: 0 8px 24px rgba(4, 8, 24, 0.18), inset 0 1px rgba(255, 255, 255, 0.06);
  }
  .head {
    margin-bottom: 10px;
  }
  h3 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
  }
  .body {
    color: var(--glass-text-dim);
    font-size: 13px;
    line-height: 1.5;
  }
  .foot {
    margin-top: 14px;
    padding-top: 12px;
    border-top: 1px solid var(--glass-border);
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
