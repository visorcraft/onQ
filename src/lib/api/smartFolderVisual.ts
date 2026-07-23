import { invoke } from '@tauri-apps/api/core';

export type VisualPredicate = {
  field: string;
  op: string;
  value: string;
};

export type VisualQuery = {
  predicates: VisualPredicate[];
};

export async function visualToDsl(visual: VisualQuery): Promise<string> {
  return invoke<string>('visual_to_dsl', { visual });
}

export async function dslToVisual(dsl: string): Promise<VisualQuery | null> {
  return invoke<VisualQuery | null>('dsl_to_visual', { dsl });
}
