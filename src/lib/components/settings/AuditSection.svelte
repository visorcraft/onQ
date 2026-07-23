<script lang="ts">
  import { readAuditLog, type AuditEvent } from '$lib/api/audit';

  let events = $state<AuditEvent[]>([]);
  let error = $state<string | null>(null);
  let loading = $state(false);

  async function refresh() {
    loading = true;
    error = null;
    try {
      events = await readAuditLog(80);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    void refresh();
  });
</script>

<section class="panel" aria-labelledby="audit-heading">
  <div class="panel-head">
    <h3 id="audit-heading">Security audit log</h3>
    <p class="help">
      Local JSONL under <code>.onq/audit.log</code>. Unlock, lock, import, export, and history restore
      are recorded.
    </p>
  </div>
  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
  <div class="row-actions">
    <button type="button" class="control-btn" disabled={loading} onclick={() => void refresh()}>
      {loading ? 'Loading…' : 'Refresh'}
    </button>
  </div>
  {#if events.length === 0}
    <p class="hint">No audit events yet.</p>
  {:else}
    <ul class="audit-list">
      {#each events as ev, i (`${ev.at}-${ev.kind}-${i}`)}
        <li>
          <span class="kind">{ev.kind}</span>
          <span class="at mono">{ev.at}</span>
          {#if ev.detail}
            <span class="detail mono">{ev.detail}</span>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .audit-list {
    list-style: none;
    margin: 0.75rem 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    max-height: 16rem;
    overflow: auto;
  }
  .audit-list li {
    display: grid;
    grid-template-columns: 8rem 1fr;
    gap: 0.2rem 0.75rem;
    font-size: 0.8rem;
    padding: 0.4rem 0.5rem;
    border-radius: 8px;
    background: color-mix(in oklab, var(--surface, #1a1a22) 90%, transparent);
  }
  .kind {
    font-weight: 600;
    grid-column: 1;
  }
  .at {
    grid-column: 2;
    opacity: 0.75;
  }
  .detail {
    grid-column: 1 / -1;
    opacity: 0.85;
    word-break: break-all;
  }
  .help {
    opacity: 0.8;
    margin: 0.25rem 0 0;
  }
  .hint {
    opacity: 0.7;
  }
  .error {
    color: #f87171;
  }
  .row-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }
  .mono {
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
  }
</style>
