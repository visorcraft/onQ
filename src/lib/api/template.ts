import { invoke } from '@tauri-apps/api/core';

export type TemplateField = {
  name: string;
  default: string | null;
};

export async function parseTemplateFields(body: string): Promise<TemplateField[]> {
  return invoke<TemplateField[]>('parse_template_fields', { body });
}

export async function renderTemplateBody(
  body: string,
  values: Record<string, string>,
): Promise<string> {
  return invoke<string>('render_template_body', { body, values });
}
