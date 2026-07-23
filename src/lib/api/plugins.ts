import { invoke } from '@tauri-apps/api/core';

export type PluginInfo = {
  id: string;
  name: string;
  version: string;
  path: string;
  enabled: boolean;
  capabilities?: unknown;
  installed_at?: number;
};

export async function listPlugins(): Promise<PluginInfo[]> {
  return invoke<PluginInfo[]>('list_plugins');
}

export async function installPlugin(archivePath: string): Promise<PluginInfo> {
  return invoke<PluginInfo>('install_plugin', { archivePath });
}

export async function setPluginEnabled(id: string, enabled: boolean): Promise<void> {
  return invoke('set_plugin_enabled', { id, enabled });
}

export async function uninstallPlugin(id: string): Promise<void> {
  return invoke('uninstall_plugin', { id });
}
