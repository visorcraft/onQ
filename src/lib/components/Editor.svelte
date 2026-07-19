<script lang="ts">
  import { onMount } from 'svelte';
  import { readPrompt, savePrompt, deletePrompt } from '$lib/api/prompts';
  import { lockPrompt, unlockPrompt } from '$lib/api/lock';
  import { fly } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';

  let { id, onClose }: { id: string; onClose: () => void } = $props();

  let title = $state('');
  let body = $state('');
  let folder = $state<string | null>(null);
  let tags = $state<string[]>([]);
  let favorite = $state(false);
  let locked = $state(false);
  let charCount = $state(0);
  let loading = $state(true);

  // Display-only when locked: the user can no longer edit the body in place
  // because the authoritative copy lives in the encrypted `.enc` envelope.
  // Saving while locked would create a stale plaintext copy in the vault,
  // defeating the lock. We force `onClose()` after lock/unlock so the
  // palette reloads and the next `readPrompt` reflects the new state.
  let busy = $state(false);

  onMount(async () => {
    const p = await readPrompt(id);
    title = p.title;
    folder = p.folder;
    tags = p.tags;
    favorite = p.favorite;
    locked = p.locked;
    charCount = p.char_count;
    body = locked ? '' : '';
    loading = false;
  });

  async function save() {
    if (locked) return;
    await savePrompt({ id, title, body, folder, tags, favorite });
    onClose();
  }

  async function del() {
    if (!confirm('Delete this prompt?')) return;
    await deletePrompt(id);
    onClose();
  }

  async function toggleLock() {
    if (busy) return;
    busy = true;
    try {
      if (locked) {
        // Persist any pending plaintext edits first so unlock restores the
        // version the user actually sees on screen, not a stale copy.
        if (body.length > 0 || title.length > 0) {
          await savePrompt({ id, title, body, folder, tags, favorite });
        }
        await unlockPrompt(id);
      } else {
        // Locking snapshots the current on-disk body. Saving before locking
        // guarantees the encrypted envelope holds what the user sees.
        if (body.length > 0 || title.length > 0) {
          await savePrompt({ id, title, body, folder, tags, favorite });
        }
        await lockPrompt(id);
      }
      onClose();
    } finally {
      busy = false;
    }
  }
</script>

<div
  class="editor glass-elevated spring"
  transition:fly={{ y: 20, duration: 240, easing: quintOut }}
  role="dialog"
  aria-label="Edit prompt"
>
  {#if loading}
    <p>Loading…</p>
  {:else}
    <input class="title" bind:value={title} placeholder="Title" aria-label="Prompt title" disabled={locked} />
    <input
      class="folder"
      bind:value={folder}
      placeholder="Folder (optional)"
      aria-label="Folder"
      disabled={locked}
    />
    <textarea
      class="body"
      bind:value={body}
      oninput={(e) => (charCount = (e.target as HTMLTextAreaElement).value.length)}
      placeholder={locked ? 'Body is encrypted — unlock to edit' : 'Prompt body…'}
      aria-label="Prompt body"
      disabled={locked}
    ></textarea>
    <div class="meta">
      <span>{charCount} characters</span>
      <label><input type="checkbox" bind:checked={favorite} disabled={locked} /> Favorite</label>
      {#if locked}
        <span class="lock-badge" aria-label="locked prompt">🔒 locked</span>
      {/if}
    </div>
    <div class="actions">
      <button class="primary" onclick={save} disabled={locked || busy}>Save</button>
      <button class="danger" onclick={del} disabled={busy}>Delete</button>
      <button onclick={toggleLock} disabled={busy} aria-pressed={locked}>
        {locked ? 'Unlock' : 'Lock'}
      </button>
      <button onclick={onClose}>Cancel</button>
    </div>
  {/if}
</div>

<style>
  .editor {
    position: fixed;
    inset: 0;
    margin: auto;
    width: min(800px, 90vw);
    height: min(80vh, 700px);
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    z-index: 20;
  }
  .title {
    font-size: 24px;
    font-weight: 600;
    background: transparent;
    border: 0;
    color: var(--glass-text);
    outline: none;
  }
  .folder {
    font-size: 13px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--glass-border);
    border-radius: 8px;
    padding: 8px 10px;
    color: var(--glass-text);
  }
  .body {
    flex: 1;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--glass-border);
    border-radius: 10px;
    padding: 12px;
    color: var(--glass-text);
    font-family: 'JetBrains Mono', monospace;
    font-size: 13px;
    resize: none;
    outline: none;
  }
  .meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: var(--glass-text-dim);
    font-size: 12px;
  }
  .lock-badge {
    color: var(--glass-periwinkle);
    font-weight: 600;
  }
  .actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
  button {
    padding: 8px 16px;
    border-radius: 8px;
    border: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.06);
    color: var(--glass-text);
    cursor: pointer;
  }
  button.primary {
    background: var(--glass-accent);
    border-color: transparent;
  }
  button.danger {
    background: rgba(255, 80, 80, 0.7);
    border-color: transparent;
  }
  button:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  input:focus-visible,
  textarea:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  button:disabled,
  input:disabled,
  textarea:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>