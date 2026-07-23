<script lang="ts">
  import { onMount } from 'svelte';
  import {
    licenseDocument,
    licenseDocs,
    type LicenseDocMeta,
  } from '$lib/api/about';
  import { t, locale } from '$lib/i18n';

  let docs = $state<LicenseDocMeta[]>([]);
  let active = $state('app');
  let body = $state('');
  let filter = $state('');
  let wrap = $state(false);
  let loading = $state(true);
  let err = $state<string | null>(null);

  const activeMeta = $derived(docs.find((d) => d.id === active));
  const lineCount = $derived(body ? body.split('\n').length : 0);
  const visible = $derived.by(() => {
    if (!filter.trim()) return body;
    const needle = filter.toLowerCase();
    return body
      .split('\n')
      .filter((line) => line.toLowerCase().includes(needle))
      .join('\n');
  });

  onMount(() => {
    licenseDocs()
      .then((d) => {
        docs = d;
        if (d[0]) active = d[0].id;
      })
      .catch((e) => (err = String(e)));
  });

  $effect(() => {
    const id = active;
    if (!id) return;
    loading = true;
    licenseDocument(id)
      .then((text) => {
        body = text;
        loading = false;
      })
      .catch((e) => {
        err = String(e);
        loading = false;
      });
  });

  async function copyBody() {
    const text = visible || body;
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      /* ignore */
    }
  }
</script>

<div class="about-page licenses-page">
  <header class="row-header">
    <div class="header-grow">
      <h1>{t('about.licenses', undefined, $locale)}</h1>
      <p class="sub">{t('about.licensesSub', undefined, $locale)}</p>
    </div>
    <button type="button" class="btn-ghost sm" onclick={() => void copyBody()}
      >{t('editor.copyAction', undefined, $locale)}</button
    >
  </header>

  {#if err}
    <div class="error-banner" role="alert">{err}</div>
  {/if}

  <div class="licenses-tabs" role="tablist" aria-label={t('about.licenseDocs', undefined, $locale)}>
    {#each docs as d (d.id)}
      <button
        type="button"
        role="tab"
        class="licenses-tab"
        class:active={active === d.id}
        aria-selected={active === d.id}
        onclick={() => (active = d.id)}
      >
        {d.title}
      </button>
    {/each}
  </div>

  <div class="licenses-doc-meta">
    <div>
      <h2>{activeMeta?.title ?? '…'}</h2>
      <p>{activeMeta?.subtitle ?? ''}</p>
    </div>
    <span class="line-count">{lineCount} lines</span>
  </div>

  <div class="licenses-filter-row">
    <input
      class="filter-input"
      type="search"
      placeholder={t('about.findPlaceholder', undefined, $locale)}
      bind:value={filter}
    />
    <label class="wrap-toggle">
      <input type="checkbox" bind:checked={wrap} />
      Wrap
    </label>
    <button type="button" class="btn-ghost sm" onclick={() => (filter = '')}>Clear</button>
  </div>

  <pre class="licenses-body" class:wrap>{loading ? 'Loading…' : visible || '(no matching lines)'}</pre>

</div>

<style>
  .about-page {
    box-sizing: border-box;
    width: 100%;
    margin: 0;
    padding: 28px 28px 56px;
    color: var(--glass-text);
  }
  .row-header {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    margin-bottom: 16px;
  }
  .header-grow {
    flex: 1;
    min-width: 0;
  }
  h1 {
    margin: 0 0 4px;
    font-size: 28px;
    font-weight: 700;
  }
  h2 {
    margin: 0 0 4px;
    font-size: 18px;
  }
  .sub,
  .licenses-doc-meta p {
    margin: 0;
    color: var(--glass-text-dim);
    font-size: 13px;
  }
  .licenses-tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 6px;
    border-radius: 14px;
    border: 1px solid var(--glass-border);
    background: var(--glass-inset);
    margin-bottom: 16px;
  }
  .licenses-tab {
    appearance: none;
    border: 0;
    background: transparent;
    color: var(--glass-text-dim);
    border-radius: 10px;
    padding: 8px 12px;
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .licenses-tab.active {
    color: var(--glass-selected-fg);
    background: var(--glass-selected-bg);
    box-shadow: inset 0 -2px 0 var(--glass-selected-fg);
  }
  .licenses-doc-meta {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: flex-start;
    margin-bottom: 12px;
  }
  .line-count {
    font-size: 12px;
    color: var(--glass-text-dim);
    white-space: nowrap;
  }
  .licenses-filter-row {
    display: flex;
    gap: 10px;
    align-items: center;
    margin-bottom: 12px;
  }
  .filter-input {
    flex: 1;
    min-width: 0;
    height: 40px;
    border-radius: 999px;
    border: 1px solid var(--glass-border);
    background: var(--glass-input);
    color: var(--glass-text);
    padding: 0 16px;
    font: inherit;
  }
  .wrap-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--glass-text-dim);
    white-space: nowrap;
  }
  .licenses-body {
    margin: 0;
    max-height: 62vh;
    overflow: auto;
    padding: 16px;
    border-radius: 14px;
    border: 1px solid var(--glass-border);
    background: var(--glass-code-bg);
    color: var(--glass-code-text);
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 12px;
    line-height: 1.5;
    white-space: pre;
  }
  .licenses-body.wrap {
    white-space: pre-wrap;
    word-break: break-word;
  }
  .btn-ghost {
    appearance: none;
    border: 1px solid var(--glass-border);
    background: var(--glass-control-bg);
    color: var(--glass-text);
    border-radius: 999px;
    padding: 8px 14px;
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .btn-ghost.sm {
    padding: 6px 12px;
    font-size: 12px;
  }
  .btn-ghost:hover {
    background: var(--glass-hover-strong);
  }
  .btn-ghost:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .error-banner {
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid rgba(200, 80, 80, 0.35);
    background: rgba(200, 60, 60, 0.08);
    color: #c04040;
    font-size: 13px;
    margin-bottom: 12px;
  }
  :global(:root.dark) .error-banner {
    color: #ffb4b4;
  }
</style>
