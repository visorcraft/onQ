<script lang="ts">
  import { onMount } from 'svelte';
  import {
    creditsData,
    runtimeLicenseText,
    type CreditsData,
  } from '$lib/api/about';

  let { onBack }: { onBack: () => void } = $props();

  let data = $state<CreditsData | null>(null);
  let err = $state<string | null>(null);
  let npmFilter = $state('');
  let crateFilter = $state('');
  let licenseDialog = $state<{ title: string; body: string } | null>(null);

  const filteredPackages = $derived.by(() => {
    if (!data) return [];
    const n = npmFilter.toLowerCase();
    if (!n) return data.packages;
    return data.packages.filter(
      (p) =>
        p.name.toLowerCase().includes(n) ||
        p.license.toLowerCase().includes(n) ||
        p.role.toLowerCase().includes(n),
    );
  });

  const filteredCrates = $derived.by(() => {
    if (!data) return [];
    const n = crateFilter.toLowerCase();
    if (!n) return data.crates;
    return data.crates.filter(
      (c) =>
        c.name.toLowerCase().includes(n) || c.license.toLowerCase().includes(n),
    );
  });

  onMount(() => {
    creditsData()
      .then((d) => (data = d))
      .catch((e) => (err = String(e)));
  });

  async function openRuntimeLicense(name: string, spdxId: string) {
    if (!spdxId) {
      licenseDialog = {
        title: name,
        body: 'No bundled license text for this system runtime (provided by the host OS).',
      };
      return;
    }
    try {
      const body = await runtimeLicenseText(spdxId);
      licenseDialog = { title: name, body };
    } catch (e) {
      licenseDialog = { title: name, body: String(e) };
    }
  }

  function openUrl(url: string) {
    if (!url) return;
    window.open(url, '_blank', 'noopener,noreferrer');
  }
</script>

<div class="about-page credits-page">
  <header class="row-header">
    <button type="button" class="btn-ghost sm" onclick={onBack}>← About</button>
    <div class="header-grow">
      <h1>Credits</h1>
      <p class="sub">
        {#if data}
          {data.crateCount} Cargo crates · {data.packageCount} npm packages ·
          {data.runtimeCount} runtime components
        {:else}
          Loading inventory…
        {/if}
      </p>
    </div>
  </header>

  {#if err}
    <div class="error-banner" role="alert">{err}</div>
  {/if}

  {#if data}
    <section class="credits-runtime">
      <div class="section-head">
        <h2>Runtime components</h2>
        <p>
          System libraries the shell links against at execution. None are
          bundled — host OS / packagers provide them.
        </p>
      </div>
      <ul class="runtime-list">
        {#each data.runtime as r (r.name)}
          <li class="runtime-row">
            <div class="runtime-main">
              <div class="runtime-name">{r.name}</div>
              <div class="runtime-notes">{r.notes}</div>
            </div>
            <div class="runtime-license">{r.licenses}</div>
            <div class="runtime-actions">
              <button
                type="button"
                class="icon-btn"
                title="License text"
                aria-label="License text for {r.name}"
                onclick={() => void openRuntimeLicense(r.name, r.spdxId)}
              >
                ☰
              </button>
              <button
                type="button"
                class="icon-btn"
                title="Project site"
                aria-label="Open project for {r.name}"
                disabled={!r.projectUrl}
                onclick={() => openUrl(r.projectUrl)}
              >
                ↗
              </button>
            </div>
          </li>
        {/each}
      </ul>
    </section>

    <div class="table-section-label">NPM packages</div>
    <p class="table-hint">
      Installed JavaScript packages from the workspace lockfile (runtime UI
      plus build tooling). Full texts: Licenses → Frontend (npm).
    </p>
    <div class="filter-row">
      <input
        class="filter-input"
        type="search"
        placeholder="Filter by package name, role, or license…"
        bind:value={npmFilter}
      />
      <span class="count">{filteredPackages.length} / {data.packageCount}</span>
    </div>
    <div class="table-wrap">
      <table class="credits-table">
        <thead>
          <tr>
            <th>Package</th>
            <th>Version</th>
            <th>Role</th>
            <th>License expression</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each filteredPackages as p (p.name)}
            <tr>
              <td class="mono name">{p.name}</td>
              <td class="mono">{p.version}</td>
              <td><span class="chip role">{p.role}</span></td>
              <td><span class="chip license">{p.license}</span></td>
              <td>
                {#if p.repository}
                  <button
                    type="button"
                    class="icon-btn"
                    aria-label="Open {p.name}"
                    onclick={() => openUrl(p.repository)}
                  >
                    ↗
                  </button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="table-section-label">Cargo crates</div>
    <p class="table-hint">
      Every direct and transitive Rust crate. Full texts: Licenses →
      Third-party (Rust).
    </p>
    <div class="filter-row">
      <input
        class="filter-input"
        type="search"
        placeholder="Filter by crate name or license…"
        bind:value={crateFilter}
      />
      <span class="count">{filteredCrates.length} / {data.crateCount}</span>
    </div>
    <div class="table-wrap">
      <table class="credits-table">
        <thead>
          <tr>
            <th>Crate</th>
            <th>Version</th>
            <th>License expression</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each filteredCrates as c (c.name + c.version)}
            <tr>
              <td class="mono name">{c.name}</td>
              <td class="mono">{c.version}</td>
              <td><span class="chip license">{c.license}</span></td>
              <td>
                {#if c.repository}
                  <button
                    type="button"
                    class="icon-btn"
                    aria-label="Open {c.name}"
                    onclick={() => openUrl(c.repository)}
                  >
                    ↗
                  </button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

{#if licenseDialog}
  <div class="dialog-backdrop">
    <div
      class="license-dialog"
      role="dialog"
      aria-modal="true"
      aria-label={licenseDialog.title}
    >
      <header class="dialog-head">
        <h3>{licenseDialog.title}</h3>
        <button type="button" class="btn-ghost sm" onclick={() => (licenseDialog = null)}
          >Close</button
        >
      </header>
      <pre class="licenses-body wrap">{licenseDialog.body}</pre>
    </div>
  </div>
{/if}

<style>
  .about-page {
    box-sizing: border-box;
    width: min(1100px, 100%);
    margin: 0 auto;
    padding: 20px 20px 48px;
    color: var(--glass-text);
  }
  .row-header {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    margin-bottom: 18px;
  }
  .header-grow {
    flex: 1;
  }
  h1 {
    margin: 0 0 4px;
    font-size: 28px;
    font-weight: 700;
  }
  h2 {
    margin: 0 0 4px;
    font-size: 16px;
  }
  .sub,
  .section-head p,
  .table-hint {
    margin: 0;
    color: var(--glass-text-dim);
    font-size: 13px;
  }
  .credits-runtime {
    border: 1px solid var(--glass-border);
    border-radius: 14px;
    background: rgba(16, 22, 34, 0.92);
    padding: 16px;
    margin-bottom: 22px;
  }
  .section-head {
    margin-bottom: 12px;
  }
  .runtime-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .runtime-row {
    display: grid;
    grid-template-columns: 1fr minmax(140px, 220px) auto;
    gap: 12px;
    align-items: center;
    padding: 10px 8px;
    border-radius: 10px;
  }
  .runtime-row:hover {
    background: rgba(80, 220, 200, 0.04);
  }
  .runtime-name {
    font-weight: 600;
    font-size: 13px;
  }
  .runtime-notes,
  .runtime-license {
    font-size: 12px;
    color: var(--glass-text-dim);
  }
  .runtime-actions {
    display: flex;
    gap: 6px;
  }
  .table-section-label {
    margin: 18px 0 4px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--glass-text-dim);
  }
  .table-hint {
    margin-bottom: 10px;
  }
  .filter-row {
    display: flex;
    gap: 10px;
    align-items: center;
    margin-bottom: 10px;
  }
  .filter-input {
    flex: 1;
    height: 40px;
    border-radius: 999px;
    border: 1px solid var(--glass-border);
    background: rgba(12, 16, 26, 0.9);
    color: var(--glass-text);
    padding: 0 16px;
    font: inherit;
  }
  .count {
    font-size: 12px;
    color: var(--glass-text-dim);
    white-space: nowrap;
  }
  .table-wrap {
    border: 1px solid var(--glass-border);
    border-radius: 14px;
    overflow: auto;
    max-height: 42vh;
    background: rgba(12, 16, 26, 0.95);
    margin-bottom: 8px;
  }
  .credits-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }
  .credits-table th {
    position: sticky;
    top: 0;
    background: #121a28;
    text-align: left;
    padding: 10px 12px;
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--glass-text-dim);
    border-bottom: 1px solid var(--glass-border);
  }
  .credits-table td {
    padding: 9px 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }
  .credits-table tr:hover td {
    background: rgba(80, 220, 200, 0.04);
  }
  .mono {
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 12px;
  }
  .name {
    color: #9ecbff;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    border-radius: 999px;
    font-size: 11px;
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    border: 1px solid var(--glass-border);
  }
  .chip.role {
    color: #c9b8ff;
    background: rgba(160, 140, 255, 0.1);
  }
  .chip.license {
    color: #8dffc0;
    background: rgba(80, 220, 140, 0.1);
    border-color: rgba(80, 220, 140, 0.25);
  }
  .icon-btn {
    width: 32px;
    height: 32px;
    border-radius: 999px;
    border: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.04);
    color: var(--glass-text);
    cursor: pointer;
  }
  .icon-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
  }
  .icon-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
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
  .error-banner {
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid rgba(255, 100, 100, 0.35);
    background: rgba(255, 80, 80, 0.1);
    color: #ffb4b4;
    font-size: 13px;
    margin-bottom: 12px;
  }
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    z-index: 80;
    background: rgba(0, 0, 0, 0.55);
    display: grid;
    place-items: center;
    padding: 24px;
  }
  .license-dialog {
    width: min(720px, 100%);
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    border-radius: 16px;
    border: 1px solid var(--glass-border);
    background: #0e1420;
    overflow: hidden;
  }
  .dialog-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 14px;
    border-bottom: 1px solid var(--glass-border);
  }
  .dialog-head h3 {
    margin: 0;
    font-size: 15px;
  }
  .licenses-body {
    margin: 0;
    padding: 16px;
    overflow: auto;
    flex: 1;
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 12px;
    line-height: 1.5;
    color: #c9d4e4;
    white-space: pre-wrap;
  }
  @media (max-width: 800px) {
    .runtime-row {
      grid-template-columns: 1fr;
    }
  }
</style>
