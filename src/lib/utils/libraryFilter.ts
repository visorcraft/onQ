import type { PromptSummary } from '$lib/types/prompt';
import { isUnder } from '$lib/utils/folderPath';

export type LibrarySelection =
  | { kind: 'all' | 'favorites' | 'recent' | 'unfiled' }
  | { kind: 'project'; path: string }
  | { kind: 'tag'; tag: string };

/**
 * Trimmed, lowercased text filter plus the standard `updated`-descending
 * sort. `isRecent` skips the sort because the `recent` selection is already
 * slice-ordered (newest 30).
 */
export function applyTextFilterAndSort(
  list: PromptSummary[],
  filterText: string,
  isRecent: boolean,
): PromptSummary[] {
  const q = filterText.trim().toLowerCase();
  let out = list;
  if (q) {
    out = out.filter(
      (p) =>
        p.title.toLowerCase().includes(q) ||
        (p.preview ?? '').toLowerCase().includes(q) ||
        (p.tags ?? []).some((t) => t.toLowerCase().includes(q)) ||
        (p.folder ?? '').toLowerCase().includes(q),
    );
  }
  if (!isRecent) {
    out = [...out].sort((a, b) => b.updated.localeCompare(a.updated));
  }
  return out;
}

/**
 * Selection semantics for the library list. Smart-folder selection stays in
 * the component (it depends on `smartHits`); it is passed through the same
 * text-filter/sort tail via `applyTextFilterAndSort`.
 */
export function filterVisiblePrompts(
  prompts: PromptSummary[],
  selection: LibrarySelection,
  filterText: string,
): PromptSummary[] {
  let list: PromptSummary[];
  switch (selection.kind) {
    case 'all':
      list = [...prompts];
      break;
    case 'favorites':
      list = prompts.filter((p) => p.favorite);
      break;
    case 'recent':
      list = [...prompts]
        .sort((a, b) => b.updated.localeCompare(a.updated))
        .slice(0, 30);
      break;
    case 'unfiled':
      list = prompts.filter((p) => !(p.folder ?? '').trim());
      break;
    case 'project': {
      const path = selection.path;
      list = prompts.filter((p) => {
        const f = (p.folder ?? '').trim();
        return Boolean(f && isUnder(f, path));
      });
      break;
    }
    case 'tag': {
      const tag = selection.tag;
      list = prompts.filter((p) =>
        (p.tags ?? []).some((raw) => raw.trim() === tag),
      );
      break;
    }
  }
  return applyTextFilterAndSort(list, filterText, selection.kind === 'recent');
}
