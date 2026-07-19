<script lang="ts">
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
  <div class="preview" aria-label="Recovery phrase preview">
    <span>Recovery phrase</span>
    <code>{phrase}</code>
  </div>
{:else}
  <div class="backdrop">
    <div class="modal glass-elevated" role="dialog" aria-labelledby="recovery-title">
      <h2 id="recovery-title">Save your recovery phrase</h2>
      <p class="warn">
        This no-password vault opens automatically with its system keychain
        entry. This 24-word phrase is ONLY for manual recovery if that entry is
        lost. Write it down. Store it somewhere safe.
      </p>
      <textarea readonly rows="4" value={phrase} aria-label="Recovery phrase"></textarea>
      {#if copied}
        <p class="copy-status" role="status">Copied!</p>
        <label class="ack">
          <input type="checkbox" bind:checked />
          I have saved this phrase somewhere safe
        </label>
        <button disabled={!checked} onclick={onAcknowledged}>Continue</button>
      {:else}
        {#if copyFailed}<p class="copy-error" role="alert">Unable to copy. Try again.</p>{/if}
        <button onclick={() => void copyPhrase()}>Copy</button>
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
    border: 1px solid var(--glass-border);
    border-radius: 10px;
    background: rgba(0, 0, 0, 0.2);
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
    border-radius: 8px;
    border: 1px solid var(--glass-border);
    background: rgba(0, 0, 0, 0.25);
    color: var(--glass-text);
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 13px;
    resize: vertical;
  }
  textarea:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
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
