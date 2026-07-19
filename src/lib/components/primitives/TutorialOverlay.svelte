<script lang="ts">
  let {
    step = 1,
    totalSteps = 4,
    title = '',
    body = '',
    onNext,
    onSkip,
  }: {
    step?: number;
    totalSteps?: number;
    title?: string;
    body?: string;
    onNext?: () => void;
    onSkip?: () => void;
  } = $props();
</script>

<div class="backdrop" role="presentation">
  <div class="card glass-elevated spring" role="dialog" aria-modal="true" aria-label="Tutorial">
    <header>
      <p class="eyebrow">Step {step} of {totalSteps}</p>
      <h2>{title}</h2>
    </header>
    <p class="body">{body}</p>
    <footer>
      <button type="button" class="skip" onclick={onSkip}>Skip tutorial</button>
      <button type="button" class="next" onclick={onNext}>
        {step === totalSteps ? 'Finish' : 'Next'}
      </button>
    </footer>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(10, 14, 36, 0.7);
    display: grid;
    place-items: center;
    z-index: 200;
  }
  .card {
    width: min(440px, 92vw);
    padding: 28px 32px;
    color: var(--glass-text);
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .eyebrow {
    margin: 0 0 6px;
    color: var(--glass-accent);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }
  h2 {
    margin: 0;
    font-size: 22px;
    font-weight: 600;
  }
  .body {
    margin: 0;
    color: var(--glass-text-dim);
    font-size: 14px;
    line-height: 1.55;
  }
  footer {
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }
  button {
    padding: 9px 16px;
    border-radius: 9px;
    border: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.06);
    color: var(--glass-text);
    font: inherit;
    cursor: pointer;
  }
  button.next {
    background: var(--glass-accent);
    border-color: transparent;
    color: #fff;
  }
  button.skip {
    background: transparent;
    border-color: transparent;
    color: var(--glass-text-dim);
  }
  button:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
</style>
