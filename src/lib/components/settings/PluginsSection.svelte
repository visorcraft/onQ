<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import {
    installPlugin,
    listPlugins,
    setPluginEnabled,
    uninstallPlugin,
    type PluginInfo,
  } from '$lib/api/plugins';
  import { t, locale } from '$lib/i18n';

  let plugins = $state<PluginInfo[]>([]);
  let error = $state<string | null>(null);
  let busy = $state(false);

  async function refresh() {
    try {
      plugins = await listPlugins();
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  $effect(() => {
    void refresh();
  });

  async function onInstall() {
    busy = true;
    error = null;
    try {
      const path = await open({
        title: t('plugins.installTitle'),
        multiple: false,
        filters: [
          {
            name: t('plugins.filterName'),
            extensions: ['onqplugin', 'tar', 'gz', 'tgz'],
          },
        ],
      });
      if (typeof path !== 'string') return;
      await installPlugin(path);
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function toggle(p: PluginInfo) {
    try {
      await setPluginEnabled(p.id, !p.enabled);
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function remove(p: PluginInfo) {
    if (!confirm(t('plugins.uninstallConfirm', { name: p.name }))) return;
    try {
      await uninstallPlugin(p.id);
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<section class="panel" aria-labelledby="plugins-heading">
  <div class="panel-head">
    <h3 id="plugins-heading">{t('plugins.heading', undefined, $locale)}</h3>
    <p class="help">{t('plugins.help', undefined, $locale)}</p>
  </div>
  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
  <div class="row-actions">
    <button type="button" class="control-btn primary" disabled={busy} onclick={() => void onInstall()}>
      {busy
        ? t('common.installing', undefined, $locale)
        : t('common.install', undefined, $locale)}
    </button>
    <button type="button" class="control-btn" onclick={() => void refresh()}
      >{t('common.refresh', undefined, $locale)}</button
    >
  </div>
  {#if plugins.length === 0}
    <p class="hint">{t('plugins.none', undefined, $locale)}</p>
  {:else}
    <ul class="plugin-list">
      {#each plugins as p (p.id)}
        <li class="plugin-row">
          <div class="plugin-copy">
            <strong>{p.name}</strong>
            <span class="meta">{p.id} · v{p.version}</span>
          </div>
          <div class="actions">
            <button type="button" class="control-btn slim" onclick={() => void toggle(p)}>
              {p.enabled
                ? t('common.disable', undefined, $locale)
                : t('common.enable', undefined, $locale)}
            </button>
            <button type="button" class="control-btn slim" onclick={() => void remove(p)}
              >{t('common.uninstall', undefined, $locale)}</button
            >
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  /* Panel/button chrome comes from the shared settings-chrome.css (scoped
   * under .settings-page); only plugin-list styles live here. */
  .plugin-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .plugin-row {
    display: flex;
    justify-content: space-between;
    gap: 14px;
    align-items: center;
    padding: 12px 14px;
    border-radius: 12px;
    border: 1px solid var(--glass-border);
    background: var(--glass-control-bg);
    transition: border-color var(--motion-duration) ease;
  }
  .plugin-row:hover {
    border-color: var(--glass-border-strong);
  }
  .plugin-copy {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    font-size: 13px;
  }
  .meta {
    font-size: 11px;
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    color: var(--glass-text-faint);
  }
  .actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }
</style>
