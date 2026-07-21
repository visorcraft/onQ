<script lang="ts">
  let {
    label,
    variant = 'secondary',
    size = 'medium',
    disabled = false,
    type = 'button',
    onclick,
  }: {
    label: string;
    variant?: 'primary' | 'secondary' | 'danger' | 'quiet';
    size?: 'small' | 'medium' | 'large';
    disabled?: boolean;
    type?: 'button' | 'submit' | 'reset';
    onclick?: () => void;
  } = $props();
</script>

<button
  {type}
  class="btn {variant} {size}"
  {disabled}
  aria-disabled={disabled}
  onclick={onclick}
>
  {label}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border-radius: 10px;
    border: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.06);
    color: var(--glass-text);
    font: inherit;
    cursor: pointer;
    transition: background var(--motion-duration) ease, border-color var(--motion-duration) ease,
      transform var(--motion-duration) var(--motion-spring);
  }
  .btn.small {
    padding: 6px 10px;
    font-size: 12px;
  }
  .btn.medium {
    padding: 8px 16px;
    font-size: 14px;
  }
  .btn.large {
    padding: 12px 22px;
    font-size: 16px;
  }
  .btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.12);
    border-color: var(--glass-border-strong);
  }
  .btn:active:not(:disabled) {
    transform: translateY(1px);
  }
  .btn:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .btn.primary {
    background: color-mix(in srgb, var(--glass-accent) 22%, var(--glass-control-bg));
    border-color: color-mix(in srgb, var(--glass-periwinkle) 38%, var(--glass-border));
    color: var(--glass-text);
    box-shadow: none;
  }
  .btn.primary:hover:not(:disabled) {
    background: color-mix(in srgb, var(--glass-accent) 32%, var(--glass-control-bg));
    border-color: color-mix(in srgb, var(--glass-periwinkle) 55%, var(--glass-border));
  }
  .btn.danger {
    background: linear-gradient(180deg, #ff7b84 0%, #e24b56 100%);
    border-color: transparent;
    color: #fff;
    box-shadow: 0 6px 16px rgba(226, 75, 86, 0.3);
  }
  .btn.quiet {
    background: transparent;
    border-color: transparent;
    color: var(--glass-text-dim);
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
