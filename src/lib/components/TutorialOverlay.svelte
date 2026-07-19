<script lang="ts">
  import { Dialog } from 'bits-ui';
  import { tick } from 'svelte';
  import TagChip from './primitives/TagChip.svelte';
  import {
    complete,
    next,
    prev,
    tutorialStep,
    tutorialVisible,
  } from '$lib/stores/tutorial';
  import { paletteShortcut } from '$lib/shortcut';
  import { globalShortcut } from '$lib/stores/globalShortcut';

  const defaultShortcut = paletteShortcut();
  const steps = [
    {
      title: 'Open your prompt palette',
      body: `Press ${$globalShortcut || defaultShortcut} anywhere to open the palette. Search your vault, jump to a prompt, or start something new without leaving the keyboard.`,
    },
    {
      title: 'Create your first prompt',
      body: 'Open the palette and choose “New prompt.” Give it a clear title, add the prompt body, then save it to your local vault.',
    },
    {
      title: 'Add tags as you go',
      body: 'Tags keep related prompts together. Use a few specific labels so you can narrow a search without building a complicated folder tree.',
    },
    {
      title: 'Your vault stays encrypted',
      body: 'Password vaults ask for their master password when opened. No-password vaults open from the system keychain; their recovery phrase is only for manual recovery.',
    },
  ] as const;

  const current = $derived(steps[$tutorialStep]);
  const returnFocusTo =
    typeof document !== 'undefined' &&
    document.activeElement instanceof HTMLElement &&
    document.activeElement !== document.body
      ? document.activeElement
      : null;
  let busy = $state(false);

  async function finish(): Promise<void> {
    if (busy) return;
    busy = true;
    try {
      await complete();
      await tick();
      if (returnFocusTo?.isConnected) returnFocusTo.focus();
    } finally {
      busy = false;
    }
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (busy) return;
    if (event.key === 'ArrowLeft') {
      event.preventDefault();
      prev();
    } else if (event.key === 'ArrowRight' && $tutorialStep < steps.length - 1) {
      event.preventDefault();
      next();
    }
  }
</script>

<Dialog.Root
  open={$tutorialVisible}
  onOpenChange={(open) => {
    if (!open) void finish();
  }}
>
  <Dialog.Portal>
    <Dialog.Overlay class="tutorial-backdrop" />
    <Dialog.Content
      class="tutorial-card glass-elevated spring"
      onkeydown={handleKeydown}
      onEscapeKeydown={(event) => {
        event.preventDefault();
        void finish();
      }}
      onInteractOutside={(event) => {
        event.preventDefault();
        void finish();
      }}
    >
      <button
        type="button"
        class="dismiss"
        aria-label="Dismiss tutorial"
        title="Dismiss tutorial"
        disabled={busy}
        onclick={() => void finish()}
      >×</button>

      <ol class="progress" aria-label="Tutorial progress">
        {#each steps as step, index}
          <li
            class:active={index === $tutorialStep}
            class:complete={index < $tutorialStep}
            aria-current={index === $tutorialStep ? 'step' : undefined}
            aria-label={`Step ${index + 1}: ${step.title}`}
          ></li>
        {/each}
      </ol>

      <header>
        <p class="eyebrow">Step {$tutorialStep + 1} of {steps.length}</p>
        <Dialog.Title class="tutorial-title">{current.title}</Dialog.Title>
        <Dialog.Description class="tutorial-body">{current.body}</Dialog.Description>
      </header>

      <div class="demonstration">
        {#if $tutorialStep === 0}
          <div class="palette-demo" aria-hidden="true">
            <div class="palette-query">
              <span class="search-mark">⌕</span>
              <span>Search prompts…</span>
              <span class="shortcut"><kbd>{$globalShortcut || defaultShortcut}</kbd></span>
            </div>
            <div class="palette-result selected">
              <span>Release checklist</span>
              <small>workflow</small>
            </div>
            <div class="palette-result">
              <span>Support response</span>
              <small>writing</small>
            </div>
          </div>
        {:else if $tutorialStep === 1}
          <div class="create-demo" aria-hidden="true">
            <span class="create-label">Command palette</span>
            <span class="create-action"><strong>+</strong> New prompt</span>
            <span class="create-hint">Press Enter</span>
          </div>
        {:else if $tutorialStep === 2}
          <div class="tags-demo" aria-label="Example prompt tags">
            <span>Example tags</span>
            <div>
              <TagChip label="writing" active />
              <TagChip label="workflow" />
              <TagChip label="review" />
            </div>
          </div>
        {:else}
          <div class="create-demo">
            <span class="create-label">Vault encryption</span>
            <span class="create-action"><strong>✓</strong> Master password or system keychain</span>
            <span class="create-hint">You choose during vault creation</span>
          </div>
        {/if}
      </div>

      <footer>
        <button type="button" class="skip" disabled={busy} onclick={() => void finish()}>
          Skip tutorial
        </button>
        <div class="navigation">
          <button type="button" disabled={$tutorialStep === 0 || busy} onclick={prev}>Back</button>
          {#if $tutorialStep === steps.length - 1}
            <button type="button" class="primary" disabled={busy} onclick={() => void finish()}>
              {busy ? 'Saving…' : 'Done'}
            </button>
          {:else}
            <button type="button" class="primary" disabled={busy} onclick={next}>Next</button>
          {/if}
        </div>
      </footer>

      <p class="keyboard-hint">Use ← and → to move between steps</p>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<style>
  :global(.tutorial-backdrop) {
    position: fixed;
    inset: 0;
    z-index: 200;
    background:
      radial-gradient(circle at 50% 35%, rgba(91, 143, 255, 0.16), transparent 38%),
      rgba(10, 14, 36, 0.74);
  }

  :global(.tutorial-card) {
    position: fixed;
    top: 50%;
    left: 50%;
    z-index: 201;
    box-sizing: border-box;
    width: min(520px, calc(100vw - 32px));
    max-height: calc(100vh - 32px);
    padding: 28px;
    overflow-y: auto;
    color: var(--glass-text);
    transform: translate(-50%, -50%);
    box-shadow: 0 28px 80px rgba(4, 8, 24, 0.42);
  }

  .dismiss {
    position: absolute;
    top: 18px;
    right: 18px;
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    padding: 0;
    border: 1px solid transparent;
    border-radius: 50%;
    background: transparent;
    color: var(--glass-text-dim);
    font-size: 20px;
    line-height: 1;
  }

  .dismiss:hover:not(:disabled) {
    border-color: var(--glass-border);
    background: rgba(255, 255, 255, 0.08);
    color: var(--glass-text);
  }

  .progress {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
    margin: 0 46px 26px 0;
    padding: 0;
    list-style: none;
  }

  .progress li {
    height: 3px;
    border-radius: 999px;
    background: var(--glass-border-strong);
    transition: background var(--motion-duration) ease, transform var(--motion-duration) ease;
  }

  .progress li.complete {
    background: rgba(91, 143, 255, 0.55);
  }

  .progress li.active {
    background: var(--glass-periwinkle);
    transform: scaleY(1.5);
  }

  header {
    display: grid;
    gap: 8px;
  }

  .eyebrow {
    margin: 0;
    color: var(--glass-accent);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  :global(.tutorial-title) {
    margin: 0;
    padding-right: 24px;
    color: var(--glass-text);
    font-size: clamp(22px, 5vw, 28px);
    font-weight: 600;
    letter-spacing: -0.025em;
  }

  :global(.tutorial-body) {
    margin: 0;
    color: var(--glass-text-dim);
    font-size: 14px;
    line-height: 1.6;
  }

  .demonstration {
    display: grid;
    place-items: center;
    min-height: 166px;
    margin: 22px 0;
    padding: 18px;
    border: 1px solid var(--glass-border);
    border-radius: 14px;
    background: rgba(5, 10, 28, 0.22);
  }

  .palette-demo {
    width: min(100%, 380px);
    padding: 9px;
    border: 1px solid var(--glass-border-strong);
    border-radius: 12px;
    background: rgba(42, 52, 84, 0.72);
    box-shadow: 0 14px 36px rgba(4, 8, 24, 0.28);
  }

  .palette-query,
  .palette-result {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 9px 10px;
    border-radius: 8px;
  }

  .palette-query {
    margin-bottom: 5px;
    border-bottom: 1px solid var(--glass-border);
    border-radius: 6px 6px 0 0;
    color: var(--glass-text-dim);
    font-size: 13px;
  }

  .search-mark {
    color: var(--glass-periwinkle);
    font-size: 18px;
  }

  .shortcut {
    display: flex;
    gap: 3px;
    margin-left: auto;
  }

  kbd {
    min-width: 22px;
    padding: 2px 5px;
    border: 1px solid var(--glass-border);
    border-radius: 5px;
    background: rgba(255, 255, 255, 0.07);
    color: var(--glass-text);
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 10px;
    text-align: center;
  }

  .palette-result {
    justify-content: space-between;
    color: var(--glass-text);
    font-size: 12px;
  }

  .palette-result.selected {
    background: rgba(91, 143, 255, 0.2);
  }

  .palette-result small {
    color: var(--glass-text-faint);
    font-size: 10px;
  }

  .create-demo {
    display: grid;
    grid-template-columns: 1fr auto;
    align-items: center;
    width: min(100%, 360px);
    padding: 10px;
    border: 1px solid var(--glass-border-strong);
    border-radius: 12px;
    background: rgba(42, 52, 84, 0.72);
  }

  .create-label {
    grid-column: 1 / -1;
    padding: 0 8px 8px;
    color: var(--glass-text-faint);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .create-action {
    padding: 11px 12px;
    border-radius: 8px;
    background: rgba(91, 143, 255, 0.2);
    color: var(--glass-text);
    font-size: 13px;
  }

  .create-action strong {
    margin-right: 6px;
    color: var(--glass-periwinkle);
    font-size: 16px;
  }

  .create-hint {
    padding: 0 8px;
    color: var(--glass-text-faint);
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 10px;
  }

  .tags-demo {
    display: grid;
    gap: 12px;
    width: min(100%, 360px);
  }

  .tags-demo > span {
    color: var(--glass-text-faint);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .tags-demo > div {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .navigation {
    display: flex;
    gap: 8px;
  }

  button {
    padding: 9px 16px;
    border: 1px solid var(--glass-border);
    border-radius: 9px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--glass-text);
    font: inherit;
    cursor: pointer;
    transition: background var(--motion-duration) ease, transform var(--motion-duration) ease;
  }

  button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.12);
  }

  button:active:not(:disabled) {
    transform: translateY(1px);
  }

  button.primary {
    min-width: 74px;
    border-color: transparent;
    background: var(--glass-accent);
    color: #fff;
  }

  button.primary:hover:not(:disabled) {
    background: #78a3ff;
  }

  button.skip {
    padding-left: 0;
    border-color: transparent;
    background: transparent;
    color: var(--glass-text-dim);
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.45;
  }

  button:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }

  .keyboard-hint {
    margin: 16px 0 0;
    color: var(--glass-text-faint);
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 10px;
    text-align: right;
  }

  @media (max-width: 520px) {
    :global(.tutorial-card) {
      padding: 22px;
    }

    .demonstration {
      min-height: 146px;
      padding: 12px;
    }

    footer {
      align-items: stretch;
      flex-direction: column-reverse;
    }

    .navigation {
      display: grid;
      grid-template-columns: 1fr 1fr;
    }

    .skip {
      align-self: center;
    }

    .keyboard-hint {
      display: none;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .progress li,
    button {
      transition: none;
    }
  }
</style>
