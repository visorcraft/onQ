<script lang="ts">
  import { onMount } from 'svelte';
  import { listPrompts, setPromptFavorite, setPromptFolder, deletePrompt } from '$lib/api/prompts';
  import type { PromptSummary } from '$lib/types/prompt';
  import {
    listFolders,
    createFolder,
    renameFolder,
    deleteFolder,
    type Folder,
  } from '$lib/api/folders';
  import {
    listSmartFolders,
    createSmartFolder,
    updateSmartFolder,
    deleteSmartFolder,
    runSmartFolder,
    type SmartFolder,
  } from '$lib/api/smartFolders';
  import {
    applyPromptCounts,
    buildProjectTree,
    isUnder,
    leafName,
    normalizeProjectPath,
    parentPath,
    promptCountLabel,
    type ProjectTreeNode,
  } from '$lib/utils/folderPath';
  import ConfirmDialog from './primitives/ConfirmDialog.svelte';
  import Pager from './primitives/Pager.svelte';
  import SmartFolderVisualBuilder from './SmartFolderVisualBuilder.svelte';
  import type { VisualPredicate } from '$lib/api/smartFolderVisual';
  import { t, locale } from '$lib/i18n';

  /** Number of prompts shown per Library page. Tuned so a typical vault
   * (10-200 prompts) fits on a single screen on a 1080p monitor; vaults
   * above ~100 prompts get a pager without the user having to ask for it. */
  const PAGE_SIZE = 25;

  let {
    onOpenPrompt,
    onNewPrompt,
    libraryEpoch = 0,
  }: {
    // eslint-disable-next-line no-unused-vars -- type-only callback param
    onOpenPrompt: (promptId: string) => void;
    /** Open an unsaved draft editor (nothing written until Save). */
    // eslint-disable-next-line no-unused-vars -- type-only callback param
    onNewPrompt?: (folder: string | null) => void;
    /** Parent bumps this when the editor closes so the list reloads. */
    libraryEpoch?: number;
  } = $props();

  type Selection =
    | { kind: 'all' }
    | { kind: 'favorites' }
    | { kind: 'recent' }
    | { kind: 'unfiled' }
    | { kind: 'project'; path: string }
    | { kind: 'smart'; id: string }
    | { kind: 'tag'; tag: string };

  let prompts = $state<PromptSummary[]>([]);
  let folders = $state<Folder[]>([]);
  let smartFolders = $state<SmartFolder[]>([]);
  let selection = $state<Selection>({ kind: 'all' });
  let filterText = $state('');
  let currentPage = $state(1);
  let errorMessage = $state<string | null>(null);
  let busy = $state(false);
  let expanded = $state<Record<string, boolean>>({});

  // Forms
  let showNewProject = $state(false);
  let newProjectName = $state('');
  let newProjectParent = $state<string | null>(null);
  let showNewSmart = $state(false);
  let newSmartName = $state('');
  let newSmartDsl = $state('favorite:true');
  let newSmartPredicates = $state<VisualPredicate[]>([]);
  let editingSmartId = $state<string | null>(null);
  let editSmartName = $state('');
  let editSmartDsl = $state('');
  let editSmartPredicates = $state<VisualPredicate[]>([]);
  let renamingPath = $state<string | null>(null);
  let renameValue = $state('');
  let smartHits = $state<PromptSummary[] | null>(null);
  let moveMenuId = $state<string | null>(null);

  type PendingDelete =
    | { kind: 'prompt'; id: string; label: string }
    | { kind: 'project'; path: string; label: string }
    | { kind: 'smart'; id: string; label: string };

  let pendingDelete = $state<PendingDelete | null>(null);
  let confirmOpen = $state(false);

  const deleteDialog = $derived.by(() => {
    const p = pendingDelete;
    if (!p) {
      return {
        title: 'Delete?',
        description: 'This action cannot be undone.',
        itemLabel: '',
        itemKind: '',
        confirmLabel: 'Delete',
      };
    }
    switch (p.kind) {
      case 'prompt':
        return {
          title: 'Delete this prompt?',
          description: 'It will be permanently removed from your vault.',
          itemLabel: p.label,
          itemKind: 'Prompt',
          confirmLabel: 'Delete prompt',
        };
      case 'project':
        return {
          title: 'Delete this project?',
          description: 'Sub-projects are removed too. Prompts inside move to Unfiled.',
          itemLabel: p.label,
          itemKind: 'Project',
          confirmLabel: 'Delete project',
        };
      case 'smart':
        return {
          title: 'Delete this smart folder?',
          description: 'Only the saved search is removed — matching prompts stay in your vault.',
          itemLabel: p.label,
          itemKind: 'Smart folder',
          confirmLabel: 'Delete smart folder',
        };
    }
  });

  const registeredPaths = $derived.by(() => {
    const set = new Set<string>();
    for (const f of folders) set.add(f.name);
    for (const p of prompts) {
      const folder = (p.folder ?? '').trim();
      if (folder) {
        // Register full path and ancestors so the tree is complete.
        const segs = folder.split('/');
        let acc = '';
        for (let i = 0; i < segs.length; i++) {
          acc = i === 0 ? segs[0] : `${acc}/${segs[i]}`;
          set.add(acc);
        }
      }
    }
    return [...set];
  });

  const projectTree = $derived.by(() => {
    const tree = buildProjectTree(registeredPaths);
    applyPromptCounts(
      tree,
      prompts.map((p) => p.folder),
    );
    return tree;
  });

  const unfiledCount = $derived(
    prompts.filter((p) => !(p.folder ?? '').trim()).length,
  );

  const allTags = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const p of prompts) {
      for (const t of p.tags ?? []) {
        const tag = t.trim();
        if (!tag) continue;
        counts.set(tag, (counts.get(tag) ?? 0) + 1);
      }
    }
    return [...counts.entries()]
      .map(([tag, count]) => ({ tag, count }))
      .sort((a, b) => a.tag.localeCompare(b.tag));
  });

  const flatProjectPaths = $derived(
    [...registeredPaths].sort((a, b) => a.localeCompare(b)),
  );

  const selectionTitle = $derived.by(() => {
    void $locale; // re-derive when language changes
    const sel = selection;
    switch (sel.kind) {
      case 'all':
        return t('library.all');
      case 'favorites':
        return t('library.favorites');
      case 'recent':
        return t('library.recent');
      case 'unfiled':
        return t('library.unfiled');
      case 'project':
        return sel.path;
      case 'smart': {
        const sf = smartFolders.find((s) => s.id === sel.id);
        return sf?.name ?? t('library.smartFolders');
      }
      case 'tag':
        return `#${sel.tag}`;
    }
  });

  const visiblePrompts = $derived.by(() => {
    const sel = selection;
    let list: PromptSummary[];
    switch (sel.kind) {
      case 'all':
        list = [...prompts];
        break;
      case 'favorites':
        list = prompts.filter((p) => p.favorite);
        break;
      case 'recent':
        list = [...prompts].sort((a, b) => b.updated.localeCompare(a.updated)).slice(0, 30);
        break;
      case 'unfiled':
        list = prompts.filter((p) => !(p.folder ?? '').trim());
        break;
      case 'project': {
        const path = sel.path;
        list = prompts.filter((p) => {
          const f = (p.folder ?? '').trim();
          return Boolean(f && isUnder(f, path));
        });
        break;
      }
      case 'smart':
        list = smartHits ?? [];
        break;
      case 'tag': {
        const tag = sel.tag;
        list = prompts.filter((p) => (p.tags ?? []).includes(tag));
        break;
      }
    }
    const q = filterText.trim().toLowerCase();
    if (q) {
      list = list.filter(
        (p) =>
          p.title.toLowerCase().includes(q) ||
          (p.preview ?? '').toLowerCase().includes(q) ||
          (p.tags ?? []).some((t) => t.toLowerCase().includes(q)) ||
          (p.folder ?? '').toLowerCase().includes(q),
      );
    }
    if (sel.kind !== 'recent') {
      list = [...list].sort((a, b) => b.updated.localeCompare(a.updated));
    }
    return list;
  });

  // Pagination: page count off the un-paged visible list; the actual slice
  // clamps to a derived `safePage` so a shrink (delete, filter) never
  // flashes an empty page on a stale `currentPage` value before an effect
  // can snap it back.
  const pageCount = $derived(Math.max(1, Math.ceil(visiblePrompts.length / PAGE_SIZE)));
  const safePage = $derived(Math.min(currentPage, pageCount));
  const pagedPrompts = $derived(
    visiblePrompts.slice((safePage - 1) * PAGE_SIZE, safePage * PAGE_SIZE),
  );
  const pageRangeLabel = $derived.by(() => {
    const total = visiblePrompts.length;
    if (total === 0) return '';
    const start = (safePage - 1) * PAGE_SIZE + 1;
    const end = Math.min(total, safePage * PAGE_SIZE);
    return `Showing ${start}–${end} of ${total}`;
  });

  // Persist the clamp back into `currentPage` so prev/next clicks keep
  // working from the corrected position. Cheap because `safePage` only
  // changes when `currentPage` or `pageCount` do.
  $effect(() => {
    if (safePage !== currentPage) currentPage = safePage;
  });

  // Reset to the first page on every filter-text change. Tracked off a
  // separate `prevFilter` so the effect doesn't fire on first render.
  let prevFilter = $state('');
  $effect(() => {
    if (filterText !== prevFilter) {
      prevFilter = filterText;
      currentPage = 1;
    }
  });

  const activeSmartDsl = $derived.by(() => {
    const sel = selection;
    if (sel.kind !== 'smart') return null;
    return smartFolders.find((s) => s.id === sel.id)?.query_dsl ?? null;
  });

  onMount(() => {
    void refresh();
  });

  // Reload after editor save/delete/lock without remounting the whole page.
  $effect(() => {
    if (libraryEpoch > 0) {
      void refresh();
    }
  });

  async function refresh() {
    errorMessage = null;
    try {
      const [p, f, sf] = await Promise.all([
        listPrompts(),
        listFolders().catch(() => [] as Folder[]),
        listSmartFolders().catch(() => [] as SmartFolder[]),
      ]);
      prompts = p;
      folders = f;
      smartFolders = sf;
      if (selection.kind === 'smart') {
        await loadSmartHits(selection.id);
      }
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    }
  }

  async function loadSmartHits(id: string) {
    try {
      const hits = await runSmartFolder(id);
      const byId = new Map(prompts.map((p) => [p.id, p]));
      smartHits = hits
        .map((h) => {
          const existing = byId.get(h.id);
          if (existing) return existing;
          return {
            id: h.id,
            title: h.title,
            folder: h.folder,
            tags: h.tags,
            favorite: h.favorite,
            locked: h.locked,
            updated: new Date(h.updated_at * 1000).toISOString(),
            char_count: h.char_count,
            preview: '',
          } satisfies PromptSummary;
        })
        .filter(Boolean);
    } catch (e) {
      smartHits = [];
      errorMessage = e instanceof Error ? e.message : String(e);
    }
  }

  async function select(next: Selection) {
    selection = next;
    filterText = '';
    currentPage = 1;
    smartHits = null;
    moveMenuId = null;
    if (next.kind === 'smart') {
      await loadSmartHits(next.id);
    }
  }

  function toggleExpand(path: string) {
    expanded = { ...expanded, [path]: !expanded[path] };
  }

  function isExpanded(path: string): boolean {
    // Default: expand depth-1 roots
    if (expanded[path] !== undefined) return expanded[path];
    return !path.includes('/');
  }

  async function addProject() {
    const raw = newProjectName.trim();
    if (!raw) return;
    busy = true;
    errorMessage = null;
    try {
      const full = newProjectParent
        ? normalizeProjectPath(`${newProjectParent}/${raw}`)
        : normalizeProjectPath(raw);
      await createFolder(full);
      newProjectName = '';
      showNewProject = false;
      newProjectParent = null;
      await refresh();
      await select({ kind: 'project', path: full });
      expanded = { ...expanded, ...(parentPath(full) ? { [parentPath(full)!]: true } : {}) };
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function commitRename() {
    if (!renamingPath || !renameValue.trim() || busy) {
      if (!renameValue.trim()) renamingPath = null;
      return;
    }
    const leaf = renameValue.trim();
    if (leaf.includes('/')) {
      errorMessage = 'Rename the leaf name only — use “add sub-project” for nested paths.';
      return;
    }
    const from = renamingPath;
    // Clear before await so blur + Enter cannot double-submit.
    renamingPath = null;
    busy = true;
    errorMessage = null;
    try {
      const parent = parentPath(from);
      const next = parent ? normalizeProjectPath(`${parent}/${leaf}`) : normalizeProjectPath(leaf);
      await renameFolder(from, next);
      await refresh();
      await select({ kind: 'project', path: next });
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function requestRemoveProject(path: string) {
    pendingDelete = { kind: 'project', path, label: path };
    confirmOpen = true;
  }

  async function removeProject(path: string) {
    busy = true;
    errorMessage = null;
    try {
      await deleteFolder(path);
      if (selection.kind === 'project' && isUnder(selection.path, path)) {
        selection = { kind: 'all' };
      }
      await refresh();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function addSmart() {
    if (!newSmartName.trim() || !newSmartDsl.trim()) return;
    busy = true;
    errorMessage = null;
    try {
      const sf = await createSmartFolder(newSmartName.trim(), newSmartDsl.trim());
      newSmartName = '';
      newSmartDsl = 'favorite:true';
      newSmartPredicates = [];
      showNewSmart = false;
      await refresh();
      await select({ kind: 'smart', id: sf.id });
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function saveSmartEdit() {
    if (!editingSmartId) return;
    busy = true;
    errorMessage = null;
    try {
      await updateSmartFolder(editingSmartId, editSmartName.trim(), editSmartDsl.trim());
      editingSmartId = null;
      await refresh();
      if (selection.kind === 'smart') await loadSmartHits(selection.id);
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function requestRemoveSmart(sf: SmartFolder) {
    pendingDelete = { kind: 'smart', id: sf.id, label: sf.name };
    confirmOpen = true;
  }

  async function removeSmart(id: string) {
    busy = true;
    errorMessage = null;
    try {
      await deleteSmartFolder(id);
      if (selection.kind === 'smart' && selection.id === id) {
        selection = { kind: 'all' };
        smartHits = null;
      }
      await refresh();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function newPromptHere() {
    // Do not create a vault entry until the user hits Save in the editor.
    const folder = selection.kind === 'project' ? selection.path : null;
    if (onNewPrompt) {
      onNewPrompt(folder);
      return;
    }
    // Fallback: open via parent open handler with a draft signal if only
    // onOpenPrompt is wired (should not happen in the shipped app).
    errorMessage = 'Unable to open new prompt editor.';
  }

  async function toggleFavorite(p: PromptSummary) {
    if (busy) return;
    busy = true;
    errorMessage = null;
    try {
      const updated = await setPromptFavorite(p.id, !p.favorite);
      prompts = prompts.map((x) =>
        x.id === updated.id ? { ...x, favorite: updated.favorite } : x,
      );
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function moveTo(p: PromptSummary, folder: string | null) {
    if (busy) return;
    busy = true;
    errorMessage = null;
    moveMenuId = null;
    try {
      const updated = await setPromptFolder(p.id, folder);
      prompts = prompts.map((x) =>
        x.id === updated.id ? { ...x, folder: updated.folder } : x,
      );
      if (selection.kind === 'smart') await loadSmartHits(selection.id);
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function requestRemovePrompt(p: PromptSummary) {
    if (busy) return;
    pendingDelete = { kind: 'prompt', id: p.id, label: p.title || 'Untitled' };
    confirmOpen = true;
  }

  async function removePrompt(id: string) {
    busy = true;
    errorMessage = null;
    try {
      await deletePrompt(id);
      prompts = prompts.filter((x) => x.id !== id);
      if (smartHits) smartHits = smartHits.filter((x) => x.id !== id);
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function executePendingDelete() {
    const p = pendingDelete;
    if (!p) return;
    try {
      if (p.kind === 'prompt') await removePrompt(p.id);
      else if (p.kind === 'project') await removeProject(p.path);
      else await removeSmart(p.id);
    } finally {
      confirmOpen = false;
      pendingDelete = null;
    }
  }

  function cancelPendingDelete() {
    confirmOpen = false;
    pendingDelete = null;
  }

  function startRename(path: string) {
    renamingPath = path;
    renameValue = leafName(path);
  }

  function startEditSmart(sf: SmartFolder) {
    editingSmartId = sf.id;
    editSmartName = sf.name;
    editSmartDsl = sf.query_dsl;
    editSmartPredicates = [];
    void import('$lib/api/smartFolderVisual').then(({ dslToVisual }) =>
      dslToVisual(sf.query_dsl).then((v) => {
        if (v?.predicates) editSmartPredicates = v.predicates;
      }),
    );
  }

  function openNewChild(parent: string | null) {
    newProjectParent = parent;
    newProjectName = '';
    showNewProject = true;
  }
</script>

<div class="library">
  <div class="page-glow" aria-hidden="true"></div>
  <header class="lib-top">
    <div class="lib-top-main">
      <p class="eyebrow">Vault</p>
      <h1>Library</h1>
      <p class="sub">Browse projects, review prompts, and run smart folders.</p>
    </div>
    <button type="button" class="control-btn primary" disabled={busy} onclick={() => void newPromptHere()}>
      <span class="plus" aria-hidden="true">+</span>
      New prompt
    </button>
  </header>

  {#if errorMessage}
    <p class="error banner" role="alert">{errorMessage}</p>
  {/if}

  <div class="lib-layout">
    <aside class="sidebar" aria-label={t('library.nav', undefined, $locale)}>
      <div class="nav-group">
        <div class="nav-label">{t('library.promptList', undefined, $locale)}</div>
        <button
          type="button"
          class="nav-item"
          class:active={selection.kind === 'all'}
          onclick={() => void select({ kind: 'all' })}
        >
          <span>{t('library.all', undefined, $locale)}</span>
          <span class="count">{prompts.length}</span>
        </button>
        <button
          type="button"
          class="nav-item"
          class:active={selection.kind === 'favorites'}
          onclick={() => void select({ kind: 'favorites' })}
        >
          <span>{t('library.favorites', undefined, $locale)}</span>
          <span class="count">{prompts.filter((p) => p.favorite).length}</span>
        </button>
        <button
          type="button"
          class="nav-item"
          class:active={selection.kind === 'recent'}
          onclick={() => void select({ kind: 'recent' })}
        >
          <span>{t('library.recent', undefined, $locale)}</span>
        </button>
        <button
          type="button"
          class="nav-item"
          class:active={selection.kind === 'unfiled'}
          onclick={() => void select({ kind: 'unfiled' })}
        >
          <span>{t('library.unfiled', undefined, $locale)}</span>
          <span class="count">{unfiledCount}</span>
        </button>
      </div>

      <div class="nav-group">
        <div class="nav-label-row">
          <div class="nav-label">{t('library.projects', undefined, $locale)}</div>
          <button
            type="button"
            class="icon-sm"
            title={t('library.newProject', undefined, $locale)}
            aria-label={t('library.newProject', undefined, $locale)}
            onclick={() => openNewChild(null)}>+</button
          >
        </div>

        {#if showNewProject}
          <div class="inline-create">
            {#if newProjectParent}
              <div class="hint">Under {newProjectParent}</div>
            {/if}
            <input
              type="text"
              placeholder={newProjectParent
                ? t('library.subProjectName', undefined, $locale)
                : t('library.projectName', undefined, $locale)}
              bind:value={newProjectName}
              onkeydown={(e) => e.key === 'Enter' && void addProject()}
            />
            <div class="row-actions">
              <button type="button" class="control-btn sm" disabled={busy || !newProjectName.trim()} onclick={() => void addProject()}
                >{t('library.create', undefined, $locale)}</button
              >
              <button
                type="button"
                class="btn-ghost sm"
                onclick={() => {
                  showNewProject = false;
                  newProjectParent = null;
                }}>{t('library.cancel', undefined, $locale)}</button
              >
            </div>
          </div>
        {/if}

        {#snippet projectNode(node: ProjectTreeNode, depth: number)}
          {@const active =
            selection.kind === 'project' && selection.path === node.path}
          <div class="tree-row" style:--d={depth}>
            {#if node.children.length > 0}
              <button
                type="button"
                class="twist"
                aria-label={isExpanded(node.path) ? 'Collapse' : 'Expand'}
                onclick={() => toggleExpand(node.path)}
              >
                {isExpanded(node.path) ? '▾' : '▸'}
              </button>
            {:else}
              <span class="twist-spacer"></span>
            {/if}
            {#if renamingPath === node.path}
              <input
                class="rename-input"
                bind:value={renameValue}
                onkeydown={(e) => {
                  if (e.key === 'Enter') void commitRename();
                  if (e.key === 'Escape') renamingPath = null;
                }}
                onblur={() => void commitRename()}
              />
            {:else}
              <button
                type="button"
                class="nav-item tree"
                class:active
                onclick={() => void select({ kind: 'project', path: node.path })}
              >
                <span class="tree-name" title={node.path}>{node.name}</span>
                <span class="count">{node.count}</span>
              </button>
              <div class="tree-actions">
                <button
                  type="button"
                  class="icon-sm"
                  title="Add sub-project"
                  aria-label="Add sub-project under {node.name}"
                  onclick={() => openNewChild(node.path)}>+</button
                >
                <button
                  type="button"
                  class="icon-sm"
                  title="Rename"
                  aria-label="Rename {node.name}"
                  onclick={() => startRename(node.path)}>✎</button
                >
                <button
                  type="button"
                  class="icon-sm danger"
                  title="Delete"
                  aria-label="Delete {node.name}"
                  onclick={() => requestRemoveProject(node.path)}>×</button
                >
              </div>
            {/if}
          </div>
          {#if isExpanded(node.path)}
            {#each node.children as child (child.path)}
              {@render projectNode(child, depth + 1)}
            {/each}
          {/if}
        {/snippet}

        {#if projectTree.length === 0}
          <p class="empty-side">{t('library.noProjects', undefined, $locale)}</p>
        {:else}
          {#each projectTree as node (node.path)}
            {@render projectNode(node, 0)}
          {/each}
        {/if}
      </div>

      <div class="nav-group">
        <div class="nav-label-row">
          <div class="nav-label">{t('library.smartFolders', undefined, $locale)}</div>
          <button
            type="button"
            class="icon-sm"
            title={t('library.newSmart', undefined, $locale)}
            aria-label={t('library.newSmart', undefined, $locale)}
            onclick={() => (showNewSmart = true)}>+</button
          >
        </div>
        {#if showNewSmart}
          <div class="inline-create">
            <input
              type="text"
              placeholder={t('library.name', undefined, $locale)}
              bind:value={newSmartName}
            />
            <SmartFolderVisualBuilder
              bind:predicates={newSmartPredicates}
              onDslChange={(dsl) => (newSmartDsl = dsl)}
            />
            <input
              type="text"
              class="mono"
              placeholder='DSL e.g. favorite:true  tag:writing'
              bind:value={newSmartDsl}
            />
            <p class="hint">
              Build with chips or edit DSL: <code>folder:</code> <code>tag:</code>
              <code>favorite:true</code> <code>text:"…"</code>
            </p>
            <div class="row-actions">
              <button type="button" class="control-btn sm" disabled={busy} onclick={() => void addSmart()}
                >{t('library.create', undefined, $locale)}</button
              >
              <button type="button" class="btn-ghost sm" onclick={() => (showNewSmart = false)}
                >{t('library.cancel', undefined, $locale)}</button
              >
            </div>
          </div>
        {/if}
        {#each smartFolders as sf (sf.id)}
          {#if editingSmartId === sf.id}
            <div class="inline-create">
              <input type="text" bind:value={editSmartName} />
              <SmartFolderVisualBuilder
                bind:predicates={editSmartPredicates}
                onDslChange={(dsl) => (editSmartDsl = dsl)}
              />
              <input type="text" class="mono" bind:value={editSmartDsl} />
              <div class="row-actions">
                <button type="button" class="control-btn sm" onclick={() => void saveSmartEdit()}
                  >{t('library.save', undefined, $locale)}</button
                >
                <button type="button" class="btn-ghost sm" onclick={() => (editingSmartId = null)}
                  >{t('library.cancel', undefined, $locale)}</button
                >
              </div>
            </div>
          {:else}
            <div class="tree-row flat">
              <button
                type="button"
                class="nav-item tree"
                class:active={selection.kind === 'smart' && selection.id === sf.id}
                onclick={() => void select({ kind: 'smart', id: sf.id })}
              >
                <span class="tree-name" title={sf.query_dsl}>{sf.name}</span>
              </button>
              <div class="tree-actions">
                <button
                  type="button"
                  class="icon-sm"
                  title="Edit"
                  aria-label="Edit {sf.name}"
                  onclick={() => startEditSmart(sf)}>✎</button
                >
                <button
                  type="button"
                  class="icon-sm danger"
                  title="Delete"
                  aria-label="Delete {sf.name}"
                  onclick={() => requestRemoveSmart(sf)}>×</button
                >
              </div>
            </div>
          {/if}
        {:else}
          {#if !showNewSmart}
            <p class="empty-side">Saved searches live here — not under Projects.</p>
          {/if}
        {/each}
      </div>

      {#if allTags.length > 0}
        <div class="nav-group">
          <div class="nav-label">{t('library.tags', undefined, $locale)}</div>
          <div class="tag-cloud">
            {#each allTags as tagEntry (tagEntry.tag)}
              <button
                type="button"
                class="tag-chip"
                class:active={selection.kind === 'tag' && selection.tag === tagEntry.tag}
                onclick={() => void select({ kind: 'tag', tag: tagEntry.tag })}
              >
                #{tagEntry.tag}
                <span class="count">{tagEntry.count}</span>
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </aside>

    <section class="main" aria-label={t('library.promptList', undefined, $locale)}>
      <div class="main-head">
        <div>
          <h2>{selectionTitle}</h2>
          <p class="meta">{promptCountLabel(visiblePrompts.length)}</p>
          {#if activeSmartDsl}
            <p class="dsl mono">{activeSmartDsl}</p>
          {/if}
        </div>
        <input
          class="filter"
          type="search"
          placeholder={t('library.filter', undefined, $locale)}
          bind:value={filterText}
          aria-label={t('library.filterAria', undefined, $locale)}
        />
      </div>

      <ul class="prompt-list">
        {#each pagedPrompts as p (p.id)}
          <li class="prompt-row">
            <button type="button" class="prompt-main" onclick={() => onOpenPrompt(p.id)}>
              <div class="prompt-title">
                {p.title || t('library.untitled', undefined, $locale)}
                {#if p.locked}<span class="lock" aria-label={t('common.locked', undefined, $locale)}>🔒</span>{/if}
              </div>
              {#if p.preview}
                <div class="prompt-preview">{p.preview}</div>
              {:else if p.locked}
                <div class="prompt-preview dim">Encrypted body — unlock to view</div>
              {:else if p.char_count === 0}
                <div class="prompt-preview dim">Empty draft</div>
              {/if}
              <div class="prompt-meta">
                {#if p.folder}
                  <span class="folder-badge">{p.folder}</span>
                {:else}
                  <span class="folder-badge muted">Unfiled</span>
                {/if}
                {#each p.tags ?? [] as tag (tag)}
                  <span class="tag-badge">#{tag}</span>
                {/each}
                <span class="chars"
                  >{p.locked ? 'encrypted' : `${p.char_count} chars`}</span
                >
              </div>
            </button>
            <div class="prompt-actions">
              <button
                type="button"
                class="icon-sm star"
                class:on={p.favorite}
                title={p.favorite ? 'Unfavorite' : 'Favorite'}
                aria-label={p.favorite ? 'Unfavorite' : 'Favorite'}
                onclick={() => void toggleFavorite(p)}
              >
                {p.favorite ? '★' : '☆'}
              </button>
              <div class="move-wrap">
                <button
                  type="button"
                  class="icon-sm"
                  title="Move to project"
                  aria-label="Move {p.title} to project"
                  onclick={() => (moveMenuId = moveMenuId === p.id ? null : p.id)}
                >
                  ↗
                </button>
                {#if moveMenuId === p.id}
                  <div class="move-menu" role="menu">
                    <button type="button" role="menuitem" onclick={() => void moveTo(p, null)}
                      >Unfiled</button
                    >
                    {#each flatProjectPaths as path (path)}
                      <button type="button" role="menuitem" onclick={() => void moveTo(p, path)}
                        >{path}</button
                      >
                    {/each}
                    {#if flatProjectPaths.length === 0}
                      <div class="hint pad">Create a project first</div>
                    {/if}
                  </div>
                {/if}
              </div>
              <button
                type="button"
                class="icon-sm danger"
                title="Delete"
                aria-label="Delete {p.title}"
                onclick={() => requestRemovePrompt(p)}
              >
                ×
              </button>
            </div>
          </li>
        {:else}
          <li class="empty-main">
            <div class="empty-icon" aria-hidden="true">◇</div>
            {#if selection.kind === 'unfiled'}
              <p>Nothing unfiled — every prompt lives in a project.</p>
            {:else if selection.kind === 'project'}
              <p>No prompts in this project yet.</p>
              <button type="button" class="control-btn primary sm" onclick={() => void newPromptHere()}
                >Create one here</button
              >
            {:else if selection.kind === 'smart'}
              <p>No prompts match this smart folder.</p>
            {:else}
              <p>No prompts yet. Create one to get started.</p>
              <button type="button" class="control-btn primary sm" onclick={() => void newPromptHere()}
                >New prompt</button
              >
            {/if}
          </li>
        {/each}
      </ul>

      <footer class="list-footer">
        <span class="range-label">{pageRangeLabel}</span>
        <Pager
          page={safePage}
          pageCount={pageCount}
          onPage={(next) => (currentPage = next)}
        />
      </footer>
    </section>
  </div>

</div>

<ConfirmDialog
  bind:open={confirmOpen}
  title={deleteDialog.title}
  description={deleteDialog.description}
  itemLabel={deleteDialog.itemLabel}
  itemKind={deleteDialog.itemKind}
  confirmLabel={deleteDialog.confirmLabel}
  busy={busy && confirmOpen}
  onConfirm={executePendingDelete}
  onCancel={cancelPendingDelete}
/>

<style>
  .library {
    position: relative;
    box-sizing: border-box;
    width: 100%;
    margin: 0;
    padding: 28px 28px 72px;
    color: var(--glass-text);
    overflow: hidden;
  }
  .page-glow {
    position: absolute;
    top: -120px;
    right: -80px;
    width: 420px;
    height: 320px;
    border-radius: 50%;
    background: radial-gradient(circle, var(--glass-accent-glow), transparent 68%);
    pointer-events: none;
  }
  .lib-top {
    position: relative;
    display: flex;
    gap: 16px;
    align-items: flex-end;
    margin-bottom: 22px;
  }
  .lib-top-main {
    flex: 1;
  }
  .eyebrow {
    margin: 0 0 6px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--glass-cyan);
  }
  h1 {
    margin: 0 0 6px;
    font-size: 32px;
    font-weight: 700;
    letter-spacing: -0.03em;
    line-height: 1.1;
  }
  h2 {
    margin: 0 0 2px;
    font-size: 18px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .sub,
  .meta {
    margin: 0;
    color: var(--glass-text-dim);
    font-size: 14px;
  }
  .dsl {
    margin: 8px 0 0;
    display: inline-block;
    padding: 4px 10px;
    border-radius: 8px;
    font-size: 12px;
    color: var(--glass-text-faint);
    background: var(--glass-control-bg);
    border: 1px solid var(--glass-border);
  }
  .lib-layout {
    position: relative;
    display: grid;
    grid-template-columns: 300px minmax(0, 1fr);
    gap: 18px;
    align-items: start;
  }
  @media (max-width: 860px) {
    .lib-layout {
      grid-template-columns: 1fr;
    }
  }
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 12px;
    border-radius: 18px;
    border: 1px solid var(--glass-border);
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.03), transparent 30%),
      var(--glass-inset);
    box-shadow: var(--glass-shadow-md), var(--glass-inset-highlight);
    position: sticky;
    top: 16px;
    max-height: calc(100vh - 100px);
    overflow: auto;
    /* Stay under modal layers; sticky can otherwise form a stacking context
     * that competes with fixed dialogs on some compositors. */
    z-index: 0;
  }
  .nav-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .nav-label,
  .nav-label-row .nav-label {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--glass-text-faint);
    padding: 6px 10px 6px;
  }
  .nav-label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .nav-item {
    appearance: none;
    border: 0;
    background: transparent;
    color: var(--glass-text-dim);
    text-align: left;
    padding: 9px 12px;
    border-radius: 11px;
    font: inherit;
    font-size: 13px;
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    gap: 8px;
    align-items: center;
    width: 100%;
    transition:
      background var(--motion-duration) ease,
      color var(--motion-duration) ease,
      box-shadow var(--motion-duration) ease;
  }
  .nav-item.tree {
    flex: 1;
    min-width: 0;
  }
  .nav-item.active {
    color: var(--glass-selected-fg);
    background: var(--glass-selected-bg);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--glass-selected-fg) 22%, transparent);
  }
  .nav-item:hover {
    background: var(--glass-hover);
    color: var(--glass-text);
  }
  .count {
    font-size: 11px;
    color: var(--glass-text-faint);
    font-variant-numeric: tabular-nums;
    min-width: 1.4em;
    text-align: right;
  }
  .tree-row {
    display: flex;
    align-items: center;
    gap: 2px;
    padding-left: calc(var(--d, 0) * 12px);
  }
  .tree-row.flat {
    padding-left: 0;
  }
  .tree-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .tree-actions {
    display: flex;
    opacity: 0;
    transition: opacity 0.12s;
  }
  .tree-row:hover .tree-actions,
  .tree-row:focus-within .tree-actions {
    opacity: 1;
  }
  .twist,
  .icon-sm {
    appearance: none;
    border: 0;
    background: transparent;
    color: var(--glass-text-dim);
    width: 26px;
    height: 26px;
    border-radius: 8px;
    cursor: pointer;
    font: inherit;
    font-size: 13px;
    line-height: 1;
    display: grid;
    place-items: center;
    flex-shrink: 0;
    transition:
      background var(--motion-duration) ease,
      color var(--motion-duration) ease;
  }
  .twist:hover,
  .icon-sm:hover {
    background: var(--glass-hover-strong);
    color: var(--glass-text);
  }
  .icon-sm.danger:hover {
    color: var(--glass-danger);
    background: var(--glass-danger-bg);
  }
  .icon-sm.star.on {
    color: var(--glass-gold);
  }
  .twist-spacer {
    width: 26px;
    flex-shrink: 0;
  }
  .inline-create {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px;
    border-radius: 12px;
    border: 1px solid var(--glass-border);
    background: color-mix(in srgb, var(--glass-panel) 70%, transparent);
    margin: 4px 0;
    box-shadow: var(--glass-inset-highlight);
  }
  .inline-create input,
  .rename-input,
  .filter {
    width: 100%;
    box-sizing: border-box;
    border-radius: 10px;
    border: 1px solid var(--glass-border);
    background: var(--glass-input);
    color: var(--glass-text);
    padding: 9px 12px;
    font: inherit;
    font-size: 13px;
    transition:
      border-color var(--motion-duration) ease,
      box-shadow var(--motion-duration) ease;
  }
  .inline-create input:focus-visible,
  .rename-input:focus-visible,
  .filter:focus-visible {
    outline: none;
    border-color: color-mix(in srgb, var(--glass-periwinkle) 55%, transparent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--glass-periwinkle) 16%, transparent);
  }
  .rename-input {
    flex: 1;
  }
  .mono,
  input.mono {
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 12px;
  }
  .hint {
    margin: 0;
    font-size: 11px;
    color: var(--glass-text-faint);
    line-height: 1.45;
  }
  .hint.pad {
    padding: 8px;
  }
  .hint code {
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 10px;
  }
  .row-actions {
    display: flex;
    gap: 6px;
  }
  .empty-side {
    margin: 4px 10px;
    font-size: 12px;
    color: var(--glass-text-faint);
    line-height: 1.45;
  }
  .tag-cloud {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 4px 6px;
  }
  .tag-chip {
    appearance: none;
    border: 1px solid var(--glass-border);
    background: var(--glass-control-bg);
    color: var(--glass-text-dim);
    border-radius: 999px;
    padding: 5px 11px;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    display: inline-flex;
    gap: 6px;
    align-items: center;
    transition:
      background var(--motion-duration) ease,
      border-color var(--motion-duration) ease,
      color var(--motion-duration) ease;
  }
  .tag-chip:hover {
    background: var(--glass-hover-strong);
    color: var(--glass-text);
  }
  .tag-chip.active {
    border-color: color-mix(in srgb, var(--glass-selected-fg) 40%, transparent);
    color: var(--glass-selected-fg);
    background: var(--glass-selected-bg);
  }
  .main {
    position: relative;
    padding: 18px;
    border-radius: 18px;
    border: 1px solid var(--glass-border);
    background:
      linear-gradient(165deg, rgba(255, 255, 255, 0.035), transparent 40%),
      var(--glass-panel);
    box-shadow: var(--glass-shadow-md), var(--glass-inset-highlight);
    min-height: 440px;
  }
  .main-head {
    display: flex;
    justify-content: space-between;
    gap: 14px;
    align-items: flex-start;
    margin-bottom: 16px;
    flex-wrap: wrap;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--glass-border);
  }
  .filter {
    width: min(280px, 100%);
  }
  .prompt-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .prompt-row {
    display: flex;
    gap: 4px;
    align-items: stretch;
    border-radius: 14px;
    border: 1px solid var(--glass-border);
    background: var(--glass-control-bg);
    transition:
      border-color var(--motion-duration) ease,
      background var(--motion-duration) ease,
      box-shadow var(--motion-duration) ease,
      transform var(--motion-duration) var(--motion-spring);
  }
  .prompt-row:hover {
    border-color: var(--glass-border-strong);
    background: color-mix(in srgb, var(--glass-control-bg) 70%, var(--glass-hover-strong));
    box-shadow: 0 8px 24px rgba(2, 6, 18, 0.18);
    transform: translateY(-1px);
  }
  .prompt-main {
    flex: 1;
    min-width: 0;
    appearance: none;
    border: 0;
    background: transparent;
    color: inherit;
    text-align: left;
    padding: 14px 16px;
    cursor: pointer;
    border-radius: 14px;
    font: inherit;
  }
  .prompt-title {
    font-weight: 650;
    font-size: 14px;
    letter-spacing: -0.01em;
    margin-bottom: 5px;
  }
  .prompt-preview {
    font-size: 12.5px;
    color: var(--glass-text-dim);
    line-height: 1.45;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    margin-bottom: 10px;
  }
  .prompt-preview.dim {
    font-style: italic;
    color: var(--glass-text-faint);
  }
  .prompt-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    font-size: 11px;
    color: var(--glass-text-faint);
  }
  .folder-badge,
  .tag-badge {
    border-radius: 999px;
    padding: 3px 9px;
    border: 1px solid var(--glass-border);
    background: color-mix(in srgb, var(--glass-panel) 80%, transparent);
  }
  .folder-badge {
    color: var(--glass-selected-fg);
    border-color: color-mix(in srgb, var(--glass-selected-fg) 28%, transparent);
    background: var(--glass-selected-bg);
  }
  .folder-badge.muted {
    color: var(--glass-text-faint);
    border-color: var(--glass-border);
    background: var(--glass-control-bg);
    opacity: 1;
  }
  .chars {
    margin-left: auto;
    font-variant-numeric: tabular-nums;
  }
  .prompt-actions {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 10px 10px 0;
    justify-content: flex-start;
  }
  .move-wrap {
    position: relative;
  }
  .move-menu {
    position: absolute;
    right: 0;
    top: 100%;
    z-index: 5;
    min-width: 190px;
    max-height: 240px;
    overflow: auto;
    border-radius: 12px;
    border: 1px solid var(--glass-border-strong);
    background: var(--glass-menu);
    box-shadow: var(--glass-shadow-md);
    display: flex;
    flex-direction: column;
    padding: 6px;
  }
  .move-menu button {
    appearance: none;
    border: 0;
    background: transparent;
    color: var(--glass-text);
    text-align: left;
    padding: 9px 11px;
    border-radius: 9px;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .move-menu button:hover {
    background: var(--glass-hover-strong);
  }
  .empty-main {
    padding: 48px 20px;
    text-align: center;
    color: var(--glass-text-dim);
    font-size: 14px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    align-items: center;
  }
  .list-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 14px;
    padding-top: 12px;
    border-top: 1px solid var(--glass-border);
    flex-wrap: wrap;
  }
  .range-label {
    font-size: 12px;
    color: var(--glass-text-dim);
    font-variant-numeric: tabular-nums;
  }
  .empty-main p {
    margin: 0;
    max-width: 48ch;
    line-height: 1.5;
  }
  .empty-icon {
    width: 52px;
    height: 52px;
    border-radius: 16px;
    display: grid;
    place-items: center;
    font-size: 20px;
    color: var(--glass-selected-fg);
    border: 1px solid color-mix(in srgb, var(--glass-selected-fg) 30%, transparent);
    background: var(--glass-selected-bg);
    box-shadow: 0 0 0 6px color-mix(in srgb, var(--glass-selected-fg) 6%, transparent);
  }
  .control-btn,
  .btn-ghost {
    appearance: none;
    border: 1px solid var(--glass-border);
    background: var(--glass-control-bg);
    color: var(--glass-text);
    border-radius: 12px;
    padding: 10px 16px;
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    transition:
      background var(--motion-duration) ease,
      border-color var(--motion-duration) ease,
      box-shadow var(--motion-duration) ease,
      transform var(--motion-duration) var(--motion-spring);
  }
  .control-btn.primary {
    border-color: color-mix(in srgb, var(--glass-periwinkle) 38%, var(--glass-border));
    background: color-mix(in srgb, var(--glass-accent) 22%, var(--glass-control-bg));
    color: var(--glass-text);
    box-shadow: none;
  }
  .control-btn.primary:hover:not(:disabled) {
    background: color-mix(in srgb, var(--glass-accent) 32%, var(--glass-control-bg));
    border-color: color-mix(in srgb, var(--glass-periwinkle) 55%, var(--glass-border));
  }
  .control-btn.sm,
  .btn-ghost.sm {
    padding: 7px 12px;
    font-size: 12px;
  }
  .plus {
    font-size: 16px;
    line-height: 1;
    font-weight: 500;
  }
  .btn-ghost {
    border-radius: 999px;
  }
  .control-btn:hover,
  .btn-ghost:hover {
    background: var(--glass-hover-strong);
  }
  .control-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .control-btn:focus-visible,
  .nav-item:focus-visible,
  .icon-sm:focus-visible,
  .tag-chip:focus-visible {
    outline: 2px solid var(--glass-periwinkle);
    outline-offset: 2px;
  }
  .error {
    margin: 0 0 14px;
    color: var(--glass-danger);
    font-size: 13px;
  }
  .error.banner {
    position: relative;
    padding: 12px 14px;
    border-radius: 12px;
    border: 1px solid var(--glass-danger-border);
    background: var(--glass-danger-bg);
  }
  .lock {
    margin-left: 6px;
  }
  @media (prefers-reduced-motion: reduce) {
    .prompt-row:hover {
      transform: none;
    }
  }
</style>
