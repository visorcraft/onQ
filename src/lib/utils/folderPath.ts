/** Hierarchical project path helpers (mirrors onq-core::folder_path). */

export const MAX_DEPTH = 8;
export const MAX_PATH_LEN = 200;

export function normalizeProjectPath(raw: string): string {
  const parts = raw
    .split('/')
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
  if (parts.length === 0) throw new Error('Project path cannot be empty');
  if (parts.some((s) => s === '.' || s === '..')) {
    throw new Error("Project path segments cannot be '.' or '..'");
  }
  if (parts.some((s) => [...s].some((ch) => {
    const code = ch.codePointAt(0) ?? 0;
    return code < 0x20 || code === 0x7f || ch === '"' || ch === "'";
  }))) {
    throw new Error('Project path contains invalid characters');
  }
  if (parts.length > MAX_DEPTH) {
    throw new Error(`Project path deeper than ${MAX_DEPTH} levels`);
  }
  const out = parts.join('/');
  if (out.length > MAX_PATH_LEN) {
    throw new Error(`Project path longer than ${MAX_PATH_LEN} characters`);
  }
  return out;
}

export function parentPath(path: string): string | null {
  const i = path.lastIndexOf('/');
  return i === -1 ? null : path.slice(0, i);
}

export function leafName(path: string): string {
  const i = path.lastIndexOf('/');
  return i === -1 ? path : path.slice(i + 1);
}

export function isUnder(child: string, ancestor: string): boolean {
  if (child === ancestor) return true;
  return child.startsWith(ancestor + '/');
}

export function depth(path: string): number {
  if (!path) return 0;
  return path.split('/').length;
}

export type ProjectTreeNode = {
  path: string;
  name: string;
  children: ProjectTreeNode[];
  /** Direct + descendant prompt counts filled by the library view. */
  count: number;
};

/** Build a tree from a flat set of project paths. */
export function buildProjectTree(paths: string[]): ProjectTreeNode[] {
  const unique = [...new Set(paths.filter(Boolean))].sort((a, b) =>
    a.localeCompare(b),
  );
  const roots: ProjectTreeNode[] = [];
  const byPath = new Map<string, ProjectTreeNode>();

  for (const path of unique) {
    const segs = path.split('/');
    let acc = '';
    for (let i = 0; i < segs.length; i++) {
      acc = i === 0 ? segs[0] : `${acc}/${segs[i]}`;
      if (byPath.has(acc)) continue;
      const node: ProjectTreeNode = {
        path: acc,
        name: segs[i],
        children: [],
        count: 0,
      };
      byPath.set(acc, node);
      const parent = parentPath(acc);
      if (parent && byPath.has(parent)) {
        byPath.get(parent)!.children.push(node);
      } else if (!parent) {
        roots.push(node);
      }
      // Ancestors are always synthesized earlier in the segment loop, so a
      // missing parent cannot occur for well-formed path lists.
    }
  }
  return roots;
}

/** Count prompts under each path (descendants included). */
export function applyPromptCounts(
  roots: ProjectTreeNode[],
  folders: (string | null | undefined)[],
): void {
  const counts = new Map<string, number>();
  for (const f of folders) {
    const path = (f ?? '').trim();
    if (!path) continue;
    // Increment every ancestor prefix.
    const segs = path.split('/');
    let acc = '';
    for (let i = 0; i < segs.length; i++) {
      acc = i === 0 ? segs[0] : `${acc}/${segs[i]}`;
      counts.set(acc, (counts.get(acc) ?? 0) + 1);
    }
  }
  const walk = (n: ProjectTreeNode) => {
    n.count = counts.get(n.path) ?? 0;
    for (const c of n.children) walk(c);
  };
  for (const r of roots) walk(r);
}

export function promptCountLabel(n: number): string {
  return n === 1 ? '1 prompt' : `${n} prompts`;
}
