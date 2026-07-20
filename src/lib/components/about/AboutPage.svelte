<script lang="ts">
  import { onMount } from 'svelte';
  import { aboutInfo, type AboutInfo } from '$lib/api/about';
  import { openExternalUrl } from '$lib/openUrl';
  import onqIcon from '../../../../crates/onq-app/icons/128x128.png';

  let {
    onBack,
    onLicenses,
    onCredits,
  }: {
    onBack?: () => void;
    onLicenses: () => void;
    onCredits: () => void;
  } = $props();

  let info = $state<AboutInfo | null>(null);
  let err = $state<string | null>(null);

  const features = [
    {
      icon: '◎',
      title: 'Hybrid search',
      body: 'Keyword + vector fusion with local MiniLM embeddings and RRF ranking.',
    },
    {
      icon: '⌘',
      title: 'Global quick access',
      body: 'Open the palette from anywhere with Win+Q, Meta+Q, or ⌘+Q.',
    },
    {
      icon: '✦',
      title: 'Encrypted Markdown vault',
      body: 'Prompts stay portable .md files; the index is encrypted at rest.',
    },
    {
      icon: '◉',
      title: 'Plugins',
      body: 'Signed Rust-native plugins over a versioned C ABI.',
    },
  ];

  onMount(() => {
    aboutInfo()
      .then((i) => (info = i))
      .catch((e) => (err = String(e)));
  });

  function openRepo() {
    const url = info?.repository ?? 'https://github.com/visorcraft/onQ';
    void openExternalUrl(url);
  }
</script>

<div class="about-page">
  {#if onBack}
    <div class="about-toolbar">
      <button type="button" class="btn-ghost" onclick={onBack}>← Back</button>
    </div>
  {/if}

  <header class="about-header">
    <h1>About</h1>
    <p class="about-header-sub">{info?.tagline ?? 'Built on Rust + Tauri 2 + Svelte 5.'}</p>
  </header>

  {#if err}
    <div class="error-banner" role="alert">{err}</div>
  {/if}

  <div class="about-body">
    <section class="about-hero">
      <div class="about-hero-halo" aria-hidden="true"></div>
      <img class="about-hero-icon" src={onqIcon} alt="onQ" width="96" height="96" draggable="false" />
      <div class="about-hero-text">
        <h2>{info?.appName ?? 'onQ'}</h2>
        <p>{info?.description ?? 'Search-oriented encrypted prompt vault.'}</p>
        <div class="about-pills">
          <span class="about-pill accent">v{info?.version ?? '…'}</span>
          <span class="about-pill">{info?.license ?? 'GPL-3.0-only'}</span>
          <span class="about-pill">{info?.platform ?? 'linux'} · Tauri 2</span>
          {#if info?.gitSha && info.gitSha !== 'unknown'}
            <span class="about-pill mono">{info.gitSha}</span>
          {/if}
        </div>
      </div>
    </section>

    <div class="about-section-label">What's inside</div>
    <div class="about-features">
      {#each features as f (f.title)}
        <div class="about-feature">
          <div class="about-feature-icon" aria-hidden="true">{f.icon}</div>
          <div>
            <div class="about-feature-title">{f.title}</div>
            <div class="about-feature-body">{f.body}</div>
          </div>
        </div>
      {/each}
    </div>

    <button type="button" class="about-link-card" onclick={openRepo}>
      <img src={onqIcon} alt="" width="40" height="40" draggable="false" />
      <div class="about-link-card-text">
        <div class="about-link-card-title">Source, issues, and releases for onQ</div>
        <div class="about-link-card-url">github.com/visorcraft/onQ</div>
      </div>
      <span class="about-link-card-cta">Visit repo →</span>
    </button>

    <section class="about-legal-card">
      <div class="about-legal-title">Licenses &amp; Credits</div>
      <p class="about-legal-body">
        Every direct + transitive Rust crate and npm package, acknowledgments,
        runtime components, and full license texts are bundled in the built-in
        licenses and credits views.
      </p>
      <div class="about-legal-actions">
        <button type="button" class="btn-ghost" onclick={onLicenses}>
          <svg class="btn-icon" viewBox="0 0 24 24" width="16" height="16" aria-hidden="true">
            <path
              fill="currentColor"
              d="M4 4h12a2 2 0 0 1 2 2v14l-4-2-4 2-4-2-4 2V6a2 2 0 0 1 2-2zm2 4h8v2H6V8zm0 4h8v2H6v-2z"
            />
          </svg>
          Licenses
        </button>
        <button type="button" class="btn-ghost" onclick={onCredits}>
          <svg class="btn-icon" viewBox="0 0 24 24" width="16" height="16" aria-hidden="true">
            <circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" stroke-width="2" />
            <circle cx="12" cy="8" r="1.25" fill="currentColor" />
            <path
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              d="M12 11.5v5"
            />
          </svg>
          Credits
        </button>
      </div>
    </section>

    <footer class="about-footer">
      Built by VisorCraft · Powered by Rust, Tauri, Svelte, and MongrelDB
    </footer>
  </div>
</div>

<style>
  .about-page {
    box-sizing: border-box;
    width: min(980px, 100%);
    margin: 0 auto;
    padding: 24px 20px 48px;
    color: var(--glass-text);
  }
  .about-toolbar {
    margin-bottom: 12px;
  }
  .about-header h1 {
    margin: 0 0 6px;
    font-size: 32px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .about-header-sub {
    margin: 0 0 20px;
    color: var(--glass-text-dim);
    font-size: 14px;
  }
  .about-body {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .about-hero {
    position: relative;
    display: flex;
    gap: 20px;
    align-items: center;
    padding: 24px;
    border-radius: 16px;
    border: 1px solid var(--glass-border);
    background: linear-gradient(135deg, rgba(16, 22, 34, 0.95), rgba(20, 32, 48, 0.9));
    overflow: hidden;
  }
  .about-hero-halo {
    position: absolute;
    inset: -40% auto auto 40%;
    width: 280px;
    height: 280px;
    border-radius: 50%;
    background: radial-gradient(circle, rgba(80, 220, 200, 0.18), transparent 65%);
    pointer-events: none;
  }
  .about-hero-icon {
    border-radius: 18px;
    flex-shrink: 0;
    z-index: 1;
  }
  .about-hero-text {
    z-index: 1;
    min-width: 0;
  }
  .about-hero-text h2 {
    margin: 0 0 6px;
    font-size: 26px;
    font-weight: 700;
  }
  .about-hero-text p {
    margin: 0 0 12px;
    color: var(--glass-text-dim);
    font-size: 14px;
  }
  .about-pills {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .about-pill {
    display: inline-flex;
    align-items: center;
    padding: 4px 10px;
    border-radius: 999px;
    border: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.04);
    font-size: 12px;
    color: var(--glass-text-dim);
  }
  .about-pill.accent {
    color: #7ee0d0;
    border-color: rgba(80, 220, 200, 0.35);
    background: rgba(80, 220, 200, 0.1);
  }
  .about-pill.mono {
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 11px;
  }
  .about-section-label {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--glass-text-dim);
  }
  .about-features {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
  }
  @media (max-width: 720px) {
    .about-features {
      grid-template-columns: 1fr;
    }
    .about-hero {
      flex-direction: column;
      align-items: flex-start;
    }
  }
  .about-feature {
    display: flex;
    gap: 12px;
    padding: 16px;
    border-radius: 14px;
    border: 1px solid var(--glass-border);
    background: rgba(16, 22, 34, 0.88);
  }
  .about-feature-icon {
    width: 40px;
    height: 40px;
    border-radius: 12px;
    display: grid;
    place-items: center;
    flex-shrink: 0;
    border: 1px solid rgba(80, 220, 200, 0.35);
    background: rgba(80, 220, 200, 0.1);
    color: #7ee0d0;
    font-size: 18px;
  }
  .about-feature-title {
    font-weight: 600;
    font-size: 14px;
    margin-bottom: 4px;
  }
  .about-feature-body {
    font-size: 12px;
    color: var(--glass-text-dim);
    line-height: 1.45;
  }
  .about-link-card {
    display: flex;
    align-items: center;
    gap: 14px;
    width: 100%;
    text-align: left;
    padding: 14px 16px;
    border-radius: 14px;
    border: 1px solid var(--glass-border);
    background: rgba(16, 22, 34, 0.88);
    color: inherit;
    cursor: pointer;
    font: inherit;
  }
  .about-link-card:hover {
    background: rgba(255, 255, 255, 0.04);
  }
  .about-link-card img {
    border-radius: 10px;
  }
  .about-link-card-text {
    flex: 1;
    min-width: 0;
  }
  .about-link-card-title {
    font-weight: 600;
    font-size: 14px;
  }
  .about-link-card-url {
    font-size: 12px;
    color: #7ee0d0;
  }
  .about-link-card-cta {
    font-size: 13px;
    color: var(--glass-text-dim);
    white-space: nowrap;
  }
  .about-legal-card {
    padding: 18px;
    border-radius: 14px;
    border: 1px solid var(--glass-border);
    background: linear-gradient(180deg, rgba(18, 26, 40, 0.95), rgba(12, 16, 26, 0.95));
  }
  .about-legal-title {
    font-weight: 700;
    font-size: 15px;
    margin-bottom: 8px;
  }
  .about-legal-body {
    margin: 0 0 14px;
    font-size: 13px;
    color: var(--glass-text-dim);
    line-height: 1.5;
  }
  .about-legal-actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }
  .btn-ghost {
    appearance: none;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    border: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.04);
    color: var(--glass-text);
    border-radius: 999px;
    padding: 8px 14px;
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .btn-ghost:hover {
    background: rgba(255, 255, 255, 0.08);
  }
  .btn-icon {
    flex-shrink: 0;
    display: block;
  }
  .about-footer {
    text-align: center;
    font-size: 12px;
    color: var(--glass-text-dim);
    padding-top: 8px;
  }
  .error-banner {
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid rgba(255, 100, 100, 0.35);
    background: rgba(255, 80, 80, 0.1);
    color: #ffb4b4;
    font-size: 13px;
  }
</style>
