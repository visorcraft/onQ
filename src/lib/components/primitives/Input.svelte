<script lang="ts">
  let {
    value = $bindable(''),
    placeholder = '',
    type = 'text',
    label = '',
    disabled = false,
    name,
  }: {
    value?: string;
    placeholder?: string;
    type?: 'text' | 'password' | 'email' | 'search' | 'url';
    label?: string;
    disabled?: boolean;
    name?: string;
  } = $props();
</script>

<label class="wrap">
  {#if label}
    <span class="lbl">{label}</span>
  {/if}
  <input
    class="field"
    {type}
    {placeholder}
    {disabled}
    {name}
    bind:value
    aria-label={label || placeholder || undefined}
  />
</label>

<style>
  .wrap {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .lbl {
    color: var(--glass-text-dim);
    font-size: 12px;
    letter-spacing: 0.02em;
  }
  .field {
    width: 100%;
    box-sizing: border-box;
    padding: 9px 12px;
    color: var(--glass-text);
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--glass-border);
    border-radius: 9px;
    font: inherit;
    font-size: 14px;
    outline: none;
    transition: border-color var(--motion-duration) ease;
  }
  .field::placeholder {
    color: var(--glass-text-faint);
  }
  .field:hover:not(:disabled) {
    border-color: var(--glass-border-strong);
  }
  .field:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 1px;
    border-color: transparent;
  }
  .field:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
