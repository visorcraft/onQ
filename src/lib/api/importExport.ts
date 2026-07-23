import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

export type ImportReport = {
  created: number;
  skipped: number;
  errors: string[];
};

export type ExportReport = {
  exported: number;
  skipped: number;
};

export async function importPrompts(
  path: string,
  format = 'auto',
  onConflict = 'skip',
): Promise<ImportReport> {
  return invoke<ImportReport>('import_prompts', { path, format, onConflict });
}

export async function exportPrompts(args: {
  dest: string;
  tagsAny?: string[];
  folder?: string | null;
  favoritesOnly?: boolean;
}): Promise<ExportReport> {
  return invoke<ExportReport>('export_prompts', {
    dest: args.dest,
    tagsAny: args.tagsAny ?? [],
    folder: args.folder ?? null,
    favoritesOnly: args.favoritesOnly ?? false,
  });
}

export async function pickImportPath(): Promise<string | null> {
  const selected = await open({
    title: 'Import prompts',
    multiple: false,
    directory: true,
  });
  if (typeof selected === 'string') return selected;
  return null;
}

export async function pickExportDir(): Promise<string | null> {
  const selected = await open({
    title: 'Export prompts to folder',
    multiple: false,
    directory: true,
  });
  if (typeof selected === 'string') return selected;
  return null;
}

