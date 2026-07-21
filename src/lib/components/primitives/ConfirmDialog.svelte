<script lang="ts">
  import { tick } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';

  let {
    open = $bindable(false),
    title = 'Delete item?',
    description = 'This action cannot be undone.',
    itemLabel = '',
    itemKind = '',
    confirmLabel = 'Delete',
    cancelLabel = 'Cancel',
    busy = false,
    onConfirm,
    onCancel,
  }: {
    open?: boolean;
    title?: string;
    description?: string;
    /** Highlighted name of the thing being deleted. */
    itemLabel?: string;
    /** Short kind label e.g. "prompt", "project", "smart folder". */
    itemKind?: string;
    confirmLabel?: string;
    cancelLabel?: string;
    busy?: boolean;
    onConfirm?: () => void | Promise<void>;
    onCancel?: () => void;
  } = $props();

  let confirmBtn = $state<HTMLButtonElement | undefined>();
  let panel = $state<HTMLDivElement | undefined>();

  $effect(() => {
    if (!open) return;
    void tick().then(() => confirmBtn?.focus());
  });

  function cancel() {
    if (busy) return;
    open = false;
    onCancel?.();
  }

  async function confirm() {
    if (busy) return;
    await onConfirm?.();
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      cancel();
    }
  }

  function onPanelKeydown(event: KeyboardEvent) {
    if (event.key !== 'Tab' || !panel) return;
    const focusable = panel.querySelectorAll<HTMLElement>(
      'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    );
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }
</script>

{#if open}
  <button
    type="button"
    class="backdrop"
    aria-label="Dismiss"
    transition:fade={{ duration: 160 }}
    onclick={cancel}
  ></button>
  <!-- Anchor keeps translate centering separate from scale transition transform. -->
  <div class="anchor">
    <div
      class="dialog glass-elevated"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="confirm-title"
      aria-describedby="confirm-desc"
      tabindex="-1"
      bind:this={panel}
      transition:scale={{ duration: 220, start: 0.94, easing: quintOut }}
      onkeydown={(e) => {
        onKeydown(e);
        onPanelKeydown(e);
      }}
    >
      <div class="glow" aria-hidden="true"></div>
      <div class="icon-wrap" aria-hidden="true">
        <svg class="icon" viewBox="0 0 24 24" width="26" height="26" fill="none">
          <path
            d="M9.5 4.5h5l.7 1.5H19a1 1 0 0 1 1 1V8H4V7a1 1 0 0 1 1-1h3.8l.7-1.5Z"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linejoin="round"
          />
          <path
            d="M6.2 8.5h11.6l-.7 10.2a1.6 1.6 0 0 1-1.6 1.5H8.5a1.6 1.6 0 0 1-1.6-1.5L6.2 8.5Z"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linejoin="round"
          />
          <path
            d="M10 11.5v5.5M14 11.5v5.5"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
          />
        </svg>
      </div>

      <div class="copy">
        {#if itemKind}
          <p class="kind">{itemKind}</p>
        {/if}
        <h2 id="confirm-title">{title}</h2>
        <p id="confirm-desc" class="desc">
          {description}
          {#if itemLabel}
            <span class="item">“{itemLabel}”</span>
          {/if}
        </p>
      </div>

      <div class="actions">
        <button type="button" class="btn cancel" disabled={busy} onclick={cancel}>
          {cancelLabel}
        </button>
        <button
          type="button"
          class="btn danger"
          bind:this={confirmBtn}
          disabled={busy}
          onclick={() => void confirm()}
        >
          {#if busy}
            Deleting…
          {:else}
            {confirmLabel}
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 120;
    border: 0;
    padding: 0;
    margin: 0;
    cursor: default;
    background:
      radial-gradient(ellipse 60% 40% at 50% 45%, rgba(240, 113, 120, 0.12), transparent 70%),
      rgba(6, 10, 22, 0.72);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
  }
  .anchor {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 121;
    width: min(420px, calc(100vw - 32px));
  }
  .dialog {
    position: relative;
    width: 100%;
    padding: 28px 28px 22px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 18px;
    text-align: center;
    color: var(--glass-text);
    overflow: hidden;
    /* Solid chrome — do not rely on translucent glass alone. */
    background: var(--glass-dialog);
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
  }
  .glow {
    position: absolute;
    top: -60%;
    left: 50%;
    width: 280px;
    height: 180px;
    transform: translateX(-50%);
    background: radial-gradient(ellipse, rgba(240, 113, 120, 0.22), transparent 70%);
    pointer-events: none;
  }
  .icon-wrap {
    position: relative;
    width: 64px;
    height: 64px;
    border-radius: 20px;
    display: grid;
    place-items: center;
    color: var(--glass-danger);
    background:
      radial-gradient(circle at 30% 25%, rgba(255, 255, 255, 0.12), transparent 55%),
      var(--glass-danger-bg);
    border: 1px solid var(--glass-danger-border);
    box-shadow:
      0 0 0 6px color-mix(in srgb, var(--glass-danger) 8%, transparent),
      0 12px 28px rgba(240, 80, 90, 0.18);
  }
  .icon {
    display: block;
  }
  .copy {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: 34ch;
  }
  .kind {
    margin: 0;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--glass-danger);
  }
  h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1.25;
  }
  .desc {
    margin: 0;
    font-size: 14px;
    line-height: 1.55;
    color: var(--glass-text-dim);
  }
  .item {
    display: inline;
    color: var(--glass-text);
    font-weight: 600;
  }
  .actions {
    position: relative;
    display: flex;
    gap: 10px;
    width: 100%;
    margin-top: 4px;
  }
  .btn {
    flex: 1;
    appearance: none;
    border-radius: 12px;
    padding: 12px 16px;
    font: inherit;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition:
      background var(--motion-duration) ease,
      border-color var(--motion-duration) ease,
      box-shadow var(--motion-duration) ease,
      transform var(--motion-duration) var(--motion-spring);
  }
  .btn:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
  .btn:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .btn.cancel {
    border: 1px solid var(--glass-border-strong);
    background: var(--glass-control-bg);
    color: var(--glass-text);
  }
  .btn.cancel:hover:not(:disabled) {
    background: var(--glass-hover-strong);
  }
  .btn.danger {
    border: 1px solid transparent;
    background: linear-gradient(180deg, #ff7b84 0%, #e24b56 100%);
    color: #fff;
    box-shadow:
      0 8px 22px rgba(226, 75, 86, 0.35),
      inset 0 1px 0 rgba(255, 255, 255, 0.25);
  }
  .btn.danger:hover:not(:disabled) {
    box-shadow:
      0 10px 28px rgba(226, 75, 86, 0.45),
      inset 0 1px 0 rgba(255, 255, 255, 0.3);
    transform: translateY(-1px);
  }
  .btn.danger:active:not(:disabled) {
    transform: translateY(0);
  }
  @media (prefers-reduced-motion: reduce) {
    .btn.danger:hover:not(:disabled) {
      transform: none;
    }
  }
</style>
