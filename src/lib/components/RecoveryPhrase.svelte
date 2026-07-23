<script lang="ts">
  import { t, locale } from '$lib/i18n';

  let {
    phrase,
    onAcknowledged,
    preview = false,
  }: {
    phrase: string;
    onAcknowledged: () => void;
    preview?: boolean;
  } = $props();

  let checked = $state(false);
  let copied = $state(false);
  let copyFailed = $state(false);

  async function copyPhrase() {
    try {
      await navigator.clipboard.writeText(phrase);
      copied = true;
    } catch {
      copyFailed = true;
    }
  }
</script>

{#if preview}
  <div class="preview" aria-label={t('recovery.previewAria', undefined, $locale)}>
    <span>{t('recovery.previewLabel', undefined, $locale)}</span>
    <code>{phrase}</code>
  </div>
{:else}
  <div class="backdrop">
    <div class="modal glass-elevated" role="dialog" aria-labelledby="recovery-title">
      <h2 id="recovery-title">{t('recovery.title', undefined, $locale)}</h2>
      <p class="warn">{t('recovery.warn', undefined, $locale)}</p>
      <textarea
        readonly
        rows="4"
        value={phrase}
        aria-label={t('recovery.aria', undefined, $locale)}
      ></textarea>
      {#if copied}
        <p class="copy-status" role="status">{t('editor.copiedAction', undefined, $locale)}</p>
        <label class="ack">
          <input type="checkbox" bind:checked />
          {t('recovery.ack', undefined, $locale)}
        </label>
        <button disabled={!checked} onclick={onAcknowledged}
          >{t('common.continue', undefined, $locale)}</button
        >
      {:else}
        {#if copyFailed}
          <p class="copy-error" role="alert">{t('recovery.copyFailed', undefined, $locale)}</p>
        {/if}
        <button onclick={() => void copyPhrase()}
          >{t('editor.copyAction', undefined, $locale)}</button
        >
      {/if}
    </div>
  </div>
{/if}

<style>
  .preview {
    display: grid;
    gap: 8px;
    width: 100%;
    padding: 14px;
    border: 1px solid var(--glass-border-strong);
    border-radius: 10px;
    background: var(--glass-input);
  }
  .preview span {
    color: var(--glass-text-faint);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }
  .preview code {
    color: var(--glass-periwinkle);
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 12px;
    line-height: 1.7;
    overflow-wrap: anywhere;
  }
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: grid;
    place-items: center;
    z-index: 100;
  }
  .modal {
    padding: 32px 40px;
    color: var(--glass-text);
    max-width: 520px;
    width: 90%;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  h2 {
    margin: 0;
    font-size: 22px;
    font-weight: 600;
  }
  .warn {
    margin: 0;
    color: var(--glass-text-dim);
    font-size: 14px;
    line-height: 1.4;
  }
  textarea {
    width: 100%;
    padding: 12px;
    border-radius: 10px;
    border: 1px solid var(--glass-border-strong);
    background: var(--glass-input);
    color: var(--glass-text);
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 13px;
    resize: vertical;
  }
  textarea:focus-visible {
    outline: none;
    border-color: color-mix(in srgb, var(--glass-periwinkle) 60%, transparent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--glass-periwinkle) 18%, transparent);
  }
  .ack {
    display: flex;
    gap: 8px;
    align-items: center;
    font-size: 14px;
  }
  .copy-status,
  .copy-error {
    margin: 0;
  }
  .copy-status {
    color: var(--glass-periwinkle);
  }
  .copy-error {
    color: #ff7a7a;
  }
  button {
    padding: 10px 20px;
    border-radius: 10px;
    border: 1px solid transparent;
    background: var(--glass-accent);
    color: #fff;
    cursor: pointer;
    font: inherit;
    align-self: flex-end;
  }
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  button:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
</style>
