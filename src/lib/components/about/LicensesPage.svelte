<script lang="ts">
  import { onMount } from 'svelte';
  import {
    licenseDocument,
    licenseDocs,
    type LicenseDocMeta,
  } from '$lib/api/about';

  let { onBack }: { onBack: () => void } = $props();

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
    <button type="button" class="btn-ghost sm" onclick={onBack}>← About</button>
    <div class="header-grow">
      <h1>Licenses</h1>
      <p class="sub">Bundled license and attribution documents, available without opening a browser.</p>
    </div>
    <button type="button" class="btn-ghost sm" onclick={() => void copyBody()}>Copy</button>
  </header>

  {#if err}
    <div class="error-banner" role="alert">{err}</div>
  {/if}

  <div class="licenses-tabs" role="tablist" aria-label="License documents">
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
      placeholder="Find by crate, package, license, or phrase…"
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
    width: min(1100px, 100%);
    margin: 0 auto;
    padding: 20px 20px 40px;
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
    background: rgba(12, 16, 26, 0.9);
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
    color: #7ee0d0;
    background: rgba(80, 220, 200, 0.12);
    box-shadow: inset 0 -2px 0 #5ad4c0;
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
    background: rgba(12, 16, 26, 0.9);
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
    background: #0a0e16;
    color: #c9d4e4;
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
    background: rgba(255, 255, 255, 0.04);
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
    background: rgba(255, 255, 255, 0.08);
  }
  .error-banner {
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid rgba(255, 100, 100, 0.35);
    background: rgba(255, 80, 80, 0.1);
    color: #ffb4b4;
    font-size: 13px;
    margin-bottom: 12px;
  }
</style>
