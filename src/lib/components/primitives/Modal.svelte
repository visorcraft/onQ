<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';

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

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      close();
    }
  }
</script>

{#if open}
  <button
    type="button"
    class="backdrop"
    aria-label="Close dialog"
    transition:fade={{ duration: 160 }}
    onclick={close}
  ></button>
  <div class="anchor">
    <div
      class="modal glass-elevated spring"
      role="dialog"
      aria-modal="true"
      aria-label={title || 'Dialog'}
      tabindex="-1"
      transition:scale={{ duration: 220, start: 0.96, easing: quintOut }}
      onkeydown={onKeydown}
    >
      <div class="modal-glow" aria-hidden="true"></div>
      {#if title}
        <header class="head">
          <h2>{title}</h2>
          <button type="button" class="close" onclick={close} aria-label="Close">
            <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
              <path
                d="M3.5 3.5l9 9M12.5 3.5l-9 9"
                stroke="currentColor"
                stroke-width="1.6"
                stroke-linecap="round"
              />
            </svg>
          </button>
        </header>
      {/if}
      <div class="body">
        {#if children}{@render children()}{/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(6, 10, 22, 0.68);
    backdrop-filter: blur(6px);
    -webkit-backdrop-filter: blur(6px);
    z-index: 100;
    border: 0;
    padding: 0;
    margin: 0;
    cursor: default;
  }
  .anchor {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 101;
    width: min(520px, 92vw);
  }
  .modal {
    position: relative;
    width: 100%;
    max-height: 86vh;
    overflow: auto;
    padding: 22px 24px;
    color: var(--glass-text);
  }
  .modal-glow {
    position: absolute;
    top: -40%;
    right: -20%;
    width: 220px;
    height: 220px;
    border-radius: 50%;
    background: radial-gradient(circle, var(--glass-accent-glow), transparent 70%);
    pointer-events: none;
  }
  .head {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 14px;
    gap: 12px;
  }
  h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .close {
    width: 32px;
    height: 32px;
    padding: 0;
    display: grid;
    place-items: center;
    background: var(--glass-control-bg);
    border: 1px solid var(--glass-border);
    border-radius: 10px;
    color: var(--glass-text-dim);
    cursor: pointer;
    flex-shrink: 0;
    transition:
      background var(--motion-duration) ease,
      color var(--motion-duration) ease;
  }
  .close:hover {
    background: var(--glass-hover-strong);
    color: var(--glass-text);
  }
  .close:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 1px;
  }
  .body {
    position: relative;
    color: var(--glass-text-dim);
    font-size: 14px;
    line-height: 1.55;
  }
</style>
