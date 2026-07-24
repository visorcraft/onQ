<script lang="ts">
  import {
    getAuditEnabled,
    readAuditLog,
    setAuditEnabled,
    type AuditEvent,
  } from '$lib/api/audit';
  import { t, locale } from '$lib/i18n';

  let events = $state<AuditEvent[]>([]);
  let error = $state<string | null>(null);
  let loading = $state(false);
  let enabled = $state(true);

  async function refresh() {
    loading = true;
    error = null;
    try {
      enabled = await getAuditEnabled();
      events = await readAuditLog(80);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function toggleEnabled(next: boolean) {
    try {
      await setAuditEnabled(next);
      enabled = next;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  $effect(() => {
    void refresh();
  });
</script>

<section class="panel" aria-labelledby="audit-heading">
  <div class="panel-head">
    <h3 id="audit-heading">{t('audit.heading', undefined, $locale)}</h3>
    <p class="help">{t('audit.help', undefined, $locale)}</p>
  </div>
  <label class="toggle-row">
    <span class="toggle-copy">
      <span class="toggle-label">{t('audit.enable', undefined, $locale)}</span>
    </span>
    <span class="switch" class:on={enabled}>
      <input
        type="checkbox"
        checked={enabled}
        onchange={(e) => void toggleEnabled(e.currentTarget.checked)}
      />
      <span class="switch-track" aria-hidden="true">
        <span class="switch-thumb"></span>
      </span>
    </span>
  </label>
  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
  <div class="toggle-row field-row">
    <span class="toggle-copy">
      <span class="toggle-label">{t('audit.heading', undefined, $locale)}</span>
    </span>
    <button type="button" class="control-btn" disabled={loading} onclick={() => void refresh()}>
      {loading
        ? t('common.loading', undefined, $locale)
        : t('common.refresh', undefined, $locale)}
    </button>
  </div>
  {#if events.length === 0}
    <p class="hint">{t('audit.none', undefined, $locale)}</p>
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
  /* Panel/button/toggle chrome comes from the shared settings-chrome.css
   * (scoped under .settings-page); only audit-list styles live here. */
  .audit-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 16rem;
    overflow: auto;
  }
  .audit-list li {
    display: grid;
    grid-template-columns: 9rem 1fr;
    gap: 2px 12px;
    padding: 8px 10px;
    border-radius: 10px;
    border: 1px solid var(--glass-border);
    background: var(--glass-control-bg);
  }
  .kind {
    grid-column: 1;
    font-size: 11px;
    font-weight: 650;
    letter-spacing: 0.02em;
    color: var(--glass-selected-fg);
  }
  .at {
    grid-column: 2;
    color: var(--glass-text-dim);
  }
  .detail {
    grid-column: 1 / -1;
    color: var(--glass-text);
    word-break: break-all;
  }
  .mono {
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 11px;
  }
</style>
