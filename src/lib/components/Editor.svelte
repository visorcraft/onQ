<script lang="ts">
  import { onMount } from 'svelte';
  import { readPrompt, savePrompt, deletePrompt } from '$lib/api/prompts';
  import { listFolders } from '$lib/api/folders';
  import { lockPrompt, unlockPrompt } from '$lib/api/lock';
  import { fly } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';

  let { id, onClose }: { id: string; onClose: () => void } = $props();

  let title = $state('');
  let body = $state('');
  let folderInput = $state('');
  let tags = $state<string[]>([]);
  let tagsInput = $state('');
  let favorite = $state(false);
  let locked = $state(false);
  let charCount = $state(0);
  let loading = $state(true);
  let projectOptions = $state<string[]>([]);
  let errorMessage = $state<string | null>(null);

  // Display-only when locked: the user can no longer edit the body in place
  // because the authoritative copy lives in the encrypted `.enc` envelope.
  // Saving while locked would create a stale plaintext copy in the vault,
  // defeating the lock. We force `onClose()` after lock/unlock so the
  // palette reloads and the next `readPrompt` reflects the new state.
  let busy = $state(false);

  onMount(async () => {
    try {
      const [p, folders] = await Promise.all([
        readPrompt(id),
        listFolders().catch(() => []),
      ]);
      title = p.title;
      folderInput = p.folder ?? '';
      tags = p.tags ?? [];
      tagsInput = tags.join(', ');
      favorite = p.favorite;
      locked = p.locked;
      charCount = p.char_count;
      body = p.body ?? '';
      projectOptions = folders.map((f) => f.name).sort((a, b) => a.localeCompare(b));
      if (p.folder && !projectOptions.includes(p.folder)) {
        projectOptions = [...projectOptions, p.folder].sort((a, b) => a.localeCompare(b));
      }
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  function parseFolder(): string | null {
    const raw = folderInput.trim();
    return raw ? raw : null;
  }

  function parseTags(): string[] {
    return tagsInput
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean);
  }

  async function save() {
    if (locked || busy) return;
    busy = true;
    errorMessage = null;
    try {
      await savePrompt({
        id,
        title,
        body,
        folder: parseFolder(),
        tags: parseTags(),
        favorite,
      });
      onClose();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function del() {
    if (!confirm('Delete this prompt?')) return;
    busy = true;
    errorMessage = null;
    try {
      await deletePrompt(id);
      onClose();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function toggleLock() {
    if (busy) return;
    busy = true;
    errorMessage = null;
    try {
      if (locked) {
        // Never call save_prompt while locked — body is sealed in `.enc`.
        await unlockPrompt(id);
      } else {
        // Persist current plaintext before sealing it.
        await savePrompt({
          id,
          title,
          body,
          folder: parseFolder(),
          tags: parseTags(),
          favorite,
        });
        await lockPrompt(id);
      }
      onClose();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function onEditorKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.stopPropagation();
      onClose();
    }
  }
</script>

<button
  type="button"
  class="editor-backdrop"
  aria-label="Close editor"
  onclick={onClose}
></button>
<div
  class="editor glass-elevated spring"
  transition:fly={{ y: 20, duration: 240, easing: quintOut }}
  role="dialog"
  aria-modal="true"
  aria-label="Edit prompt"
  tabindex="-1"
  onkeydown={onEditorKeydown}
>
  {#if loading}
    <p>Loading…</p>
  {:else}
    <input class="title" bind:value={title} placeholder="Title" aria-label="Prompt title" disabled={locked} />
    <div class="meta-row">
      <label class="field">
        <span class="field-label">Project</span>
        <input
          class="folder"
          list="project-paths"
          bind:value={folderInput}
          placeholder="Unfiled — or Writing/Blog Posts"
          aria-label="Project path"
          disabled={locked}
        />
        <datalist id="project-paths">
          {#each projectOptions as opt (opt)}
            <option value={opt}></option>
          {/each}
        </datalist>
      </label>
      <label class="field">
        <span class="field-label">Tags</span>
        <input
          class="folder"
          bind:value={tagsInput}
          placeholder="comma, separated, tags"
          aria-label="Tags"
          disabled={locked}
        />
      </label>
    </div>
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
    {#if errorMessage}
      <p class="error" role="alert">{errorMessage}</p>
    {/if}
    <div class="actions">
      <button class="primary" onclick={() => void save()} disabled={locked || busy}>Save</button>
      <button class="danger" onclick={() => void del()} disabled={busy}>Delete</button>
      <button onclick={() => void toggleLock()} disabled={busy} aria-pressed={locked}>
        {locked ? 'Unlock' : 'Lock'}
      </button>
      <button onclick={onClose}>Cancel</button>
    </div>
  {/if}
</div>

<style>
  .editor-backdrop {
    position: fixed;
    inset: 0;
    z-index: 19;
    border: 0;
    padding: 0;
    margin: 0;
    background: rgba(0, 0, 0, 0.45);
    cursor: default;
  }
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
  .meta-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  @media (max-width: 640px) {
    .meta-row {
      grid-template-columns: 1fr;
    }
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .field-label {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--glass-text-faint);
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
    gap: 12px;
  }
  .lock-badge {
    color: var(--glass-periwinkle);
    font-weight: 600;
  }
  .error {
    margin: 0;
    color: #ffb4b4;
    font-size: 13px;
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
