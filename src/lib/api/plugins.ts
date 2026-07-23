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

export type PluginCommand = {
  id: string;
  name: string;
  pluginId: string;
};

export async function listPluginCommands(): Promise<PluginCommand[]> {
  return invoke<PluginCommand[]>('list_plugin_commands');
}

export async function runPluginCommand(id: string): Promise<string> {
  return invoke<string>('run_plugin_command', { id });
}

export async function registerPluginCommand(
  id: string,
  name: string,
  pluginId: string,
): Promise<void> {
  return invoke('register_plugin_command', { id, name, pluginId });
}

export async function getEmbedderPreference(): Promise<string> {
  return invoke<string>('get_embedder_preference');
}

export async function setEmbedderPreference(id: string): Promise<void> {
  return invoke('set_embedder_preference', { id });
}
