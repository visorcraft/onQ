<script lang="ts">
  import { onMount } from 'svelte';
  import { listPrompts, createPrompt, setPromptFavorite, setPromptFolder, deletePrompt } from '$lib/api/prompts';
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

  let {
    onBack,
    onOpenPrompt,
    libraryEpoch = 0,
  }: {
    onBack: () => void;
    // eslint-disable-next-line no-unused-vars -- type-only callback param
    onOpenPrompt: (promptId: string) => void;
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
  let editingSmartId = $state<string | null>(null);
  let editSmartName = $state('');
  let editSmartDsl = $state('');
  let renamingPath = $state<string | null>(null);
  let renameValue = $state('');
  let smartHits = $state<PromptSummary[] | null>(null);
  let moveMenuId = $state<string | null>(null);

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
    const sel = selection;
    switch (sel.kind) {
      case 'all':
        return 'All prompts';
      case 'favorites':
        return 'Favorites';
      case 'recent':
        return 'Recent';
      case 'unfiled':
        return 'Unfiled';
      case 'project':
        return sel.path;
      case 'smart': {
        const sf = smartFolders.find((s) => s.id === sel.id);
        return sf?.name ?? 'Smart folder';
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

  async function removeProject(path: string) {
    if (
      !confirm(
        `Delete project “${path}” and its sub-projects?\nPrompts inside will move to Unfiled.`,
      )
    ) {
      return;
    }
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

  async function removeSmart(id: string) {
    if (!confirm('Delete this smart folder?')) return;
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

  async function newPromptHere() {
    busy = true;
    errorMessage = null;
    try {
      const folder =
        selection.kind === 'project' ? selection.path : null;
      const p = await createPrompt('Untitled', folder);
      await refresh();
      onOpenPrompt(p.id);
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
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

  async function removePrompt(p: PromptSummary) {
    if (busy) return;
    if (!confirm(`Delete prompt “${p.title || 'Untitled'}”?`)) return;
    busy = true;
    errorMessage = null;
    try {
      await deletePrompt(p.id);
      prompts = prompts.filter((x) => x.id !== p.id);
      if (smartHits) smartHits = smartHits.filter((x) => x.id !== p.id);
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function startRename(path: string) {
    renamingPath = path;
    renameValue = leafName(path);
  }

  function startEditSmart(sf: SmartFolder) {
    editingSmartId = sf.id;
    editSmartName = sf.name;
    editSmartDsl = sf.query_dsl;
  }

  function openNewChild(parent: string | null) {
    newProjectParent = parent;
    newProjectName = '';
    showNewProject = true;
  }
</script>

<div class="library">
  <header class="lib-top">
    <button type="button" class="btn-ghost" onclick={onBack}>← Back</button>
    <div class="lib-top-main">
      <h1>Library</h1>
      <p class="sub">Browse projects, review prompts, and run smart folders.</p>
    </div>
    <button type="button" class="control-btn" disabled={busy} onclick={() => void newPromptHere()}>
      + New prompt
    </button>
  </header>

  {#if errorMessage}
    <p class="error banner" role="alert">{errorMessage}</p>
  {/if}

  <div class="lib-layout">
    <aside class="sidebar" aria-label="Library navigation">
      <div class="nav-group">
        <div class="nav-label">Library</div>
        <button
          type="button"
          class="nav-item"
          class:active={selection.kind === 'all'}
          onclick={() => void select({ kind: 'all' })}
        >
          <span>All prompts</span>
          <span class="count">{prompts.length}</span>
        </button>
        <button
          type="button"
          class="nav-item"
          class:active={selection.kind === 'favorites'}
          onclick={() => void select({ kind: 'favorites' })}
        >
          <span>Favorites</span>
          <span class="count">{prompts.filter((p) => p.favorite).length}</span>
        </button>
        <button
          type="button"
          class="nav-item"
          class:active={selection.kind === 'recent'}
          onclick={() => void select({ kind: 'recent' })}
        >
          <span>Recent</span>
        </button>
        <button
          type="button"
          class="nav-item"
          class:active={selection.kind === 'unfiled'}
          onclick={() => void select({ kind: 'unfiled' })}
        >
          <span>Unfiled</span>
          <span class="count">{unfiledCount}</span>
        </button>
      </div>

      <div class="nav-group">
        <div class="nav-label-row">
          <div class="nav-label">Projects</div>
          <button
            type="button"
            class="icon-sm"
            title="New project"
            aria-label="New project"
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
              placeholder={newProjectParent ? 'Sub-project name' : 'Project name'}
              bind:value={newProjectName}
              onkeydown={(e) => e.key === 'Enter' && void addProject()}
            />
            <div class="row-actions">
              <button type="button" class="control-btn sm" disabled={busy || !newProjectName.trim()} onclick={() => void addProject()}
                >Create</button
              >
              <button
                type="button"
                class="btn-ghost sm"
                onclick={() => {
                  showNewProject = false;
                  newProjectParent = null;
                }}>Cancel</button
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
                  onclick={() => void removeProject(node.path)}>×</button
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
          <p class="empty-side">No projects yet. Create one to organize prompts.</p>
        {:else}
          {#each projectTree as node (node.path)}
            {@render projectNode(node, 0)}
          {/each}
        {/if}
      </div>

      <div class="nav-group">
        <div class="nav-label-row">
          <div class="nav-label">Smart folders</div>
          <button
            type="button"
            class="icon-sm"
            title="New smart folder"
            aria-label="New smart folder"
            onclick={() => (showNewSmart = true)}>+</button
          >
        </div>
        {#if showNewSmart}
          <div class="inline-create">
            <input type="text" placeholder="Name" bind:value={newSmartName} />
            <input
              type="text"
              class="mono"
              placeholder='DSL e.g. favorite:true  tag:writing'
              bind:value={newSmartDsl}
            />
            <p class="hint">
              Tokens: <code>folder:</code> <code>tag:</code> <code>favorite:true</code>
              <code>text:"…"</code>
            </p>
            <div class="row-actions">
              <button type="button" class="control-btn sm" disabled={busy} onclick={() => void addSmart()}
                >Create</button
              >
              <button type="button" class="btn-ghost sm" onclick={() => (showNewSmart = false)}
                >Cancel</button
              >
            </div>
          </div>
        {/if}
        {#each smartFolders as sf (sf.id)}
          {#if editingSmartId === sf.id}
            <div class="inline-create">
              <input type="text" bind:value={editSmartName} />
              <input type="text" class="mono" bind:value={editSmartDsl} />
              <div class="row-actions">
                <button type="button" class="control-btn sm" onclick={() => void saveSmartEdit()}
                  >Save</button
                >
                <button type="button" class="btn-ghost sm" onclick={() => (editingSmartId = null)}
                  >Cancel</button
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
                  onclick={() => void removeSmart(sf.id)}>×</button
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
          <div class="nav-label">Tags</div>
          <div class="tag-cloud">
            {#each allTags as t (t.tag)}
              <button
                type="button"
                class="tag-chip"
                class:active={selection.kind === 'tag' && selection.tag === t.tag}
                onclick={() => void select({ kind: 'tag', tag: t.tag })}
              >
                #{t.tag}
                <span class="count">{t.count}</span>
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </aside>

    <section class="main" aria-label="Prompt list">
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
          placeholder="Filter this list…"
          bind:value={filterText}
          aria-label="Filter prompts"
        />
      </div>

      <ul class="prompt-list">
        {#each visiblePrompts as p (p.id)}
          <li class="prompt-row">
            <button type="button" class="prompt-main" onclick={() => onOpenPrompt(p.id)}>
              <div class="prompt-title">
                {p.title || 'Untitled'}
                {#if p.locked}<span class="lock" aria-label="locked">🔒</span>{/if}
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
                onclick={() => void removePrompt(p)}
              >
                ×
              </button>
            </div>
          </li>
        {:else}
          <li class="empty-main">
            {#if selection.kind === 'unfiled'}
              Nothing unfiled — every prompt lives in a project.
            {:else if selection.kind === 'project'}
              No prompts in this project yet.
              <button type="button" class="control-btn sm" onclick={() => void newPromptHere()}
                >Create one here</button
              >
            {:else if selection.kind === 'smart'}
              No prompts match this smart folder.
            {:else}
              No prompts yet. Create one to get started.
            {/if}
          </li>
        {/each}
      </ul>
    </section>
  </div>
</div>

<style>
  .library {
    box-sizing: border-box;
    width: min(1280px, 100%);
    margin: 0 auto;
    padding: 20px 20px 48px;
    color: var(--glass-text);
  }
  .lib-top {
    display: flex;
    gap: 14px;
    align-items: flex-start;
    margin-bottom: 16px;
  }
  .lib-top-main {
    flex: 1;
  }
  h1 {
    margin: 0 0 4px;
    font-size: 28px;
    font-weight: 700;
  }
  h2 {
    margin: 0 0 2px;
    font-size: 16px;
  }
  .sub,
  .meta {
    margin: 0;
    color: var(--glass-text-dim);
    font-size: 13px;
  }
  .dsl {
    margin: 6px 0 0;
    font-size: 12px;
    color: var(--glass-text-faint);
  }
  .lib-layout {
    display: grid;
    grid-template-columns: 280px minmax(0, 1fr);
    gap: 16px;
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
    gap: 14px;
    padding: 10px;
    border-radius: 14px;
    border: 1px solid var(--glass-border);
    background: rgba(12, 16, 26, 0.92);
    position: sticky;
    top: 12px;
    max-height: calc(100vh - 100px);
    overflow: auto;
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
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--glass-text-faint);
    padding: 6px 8px 4px;
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
    padding: 8px 10px;
    border-radius: 10px;
    font: inherit;
    font-size: 13px;
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    gap: 8px;
    align-items: center;
    width: 100%;
  }
  .nav-item.tree {
    flex: 1;
    min-width: 0;
  }
  .nav-item.active {
    color: #7ee0d0;
    background: rgba(80, 220, 200, 0.12);
  }
  .nav-item:hover {
    background: rgba(255, 255, 255, 0.04);
  }
  .count {
    font-size: 11px;
    color: var(--glass-text-faint);
    font-variant-numeric: tabular-nums;
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
    width: 24px;
    height: 24px;
    border-radius: 6px;
    cursor: pointer;
    font: inherit;
    font-size: 13px;
    line-height: 1;
    display: grid;
    place-items: center;
    flex-shrink: 0;
  }
  .twist:hover,
  .icon-sm:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--glass-text);
  }
  .icon-sm.danger:hover {
    color: #ffb4b4;
  }
  .icon-sm.star.on {
    color: var(--glass-gold);
  }
  .twist-spacer {
    width: 24px;
    flex-shrink: 0;
  }
  .inline-create {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
    border-radius: 10px;
    border: 1px solid var(--glass-border);
    background: rgba(0, 0, 0, 0.2);
    margin: 4px 0;
  }
  .inline-create input,
  .rename-input,
  .filter {
    width: 100%;
    box-sizing: border-box;
    border-radius: 8px;
    border: 1px solid var(--glass-border);
    background: rgba(10, 14, 22, 0.9);
    color: var(--glass-text);
    padding: 8px 10px;
    font: inherit;
    font-size: 13px;
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
    margin: 4px 8px;
    font-size: 12px;
    color: var(--glass-text-faint);
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
    background: rgba(255, 255, 255, 0.04);
    color: var(--glass-text-dim);
    border-radius: 999px;
    padding: 4px 10px;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    display: inline-flex;
    gap: 6px;
    align-items: center;
  }
  .tag-chip.active {
    border-color: var(--glass-periwinkle);
    color: var(--glass-periwinkle);
    background: rgba(120, 163, 255, 0.12);
  }
  .main {
    padding: 16px;
    border-radius: 14px;
    border: 1px solid var(--glass-border);
    background: rgba(16, 22, 34, 0.92);
    min-height: 420px;
  }
  .main-head {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: flex-start;
    margin-bottom: 14px;
    flex-wrap: wrap;
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
    gap: 8px;
  }
  .prompt-row {
    display: flex;
    gap: 8px;
    align-items: stretch;
    border-radius: 12px;
    border: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.03);
  }
  .prompt-main {
    flex: 1;
    min-width: 0;
    appearance: none;
    border: 0;
    background: transparent;
    color: inherit;
    text-align: left;
    padding: 12px 14px;
    cursor: pointer;
    border-radius: 12px;
    font: inherit;
  }
  .prompt-main:hover {
    background: rgba(255, 255, 255, 0.04);
  }
  .prompt-title {
    font-weight: 600;
    font-size: 14px;
    margin-bottom: 4px;
  }
  .prompt-preview {
    font-size: 12px;
    color: var(--glass-text-dim);
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    margin-bottom: 8px;
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
    padding: 2px 8px;
    border: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.04);
  }
  .folder-badge.muted {
    opacity: 0.7;
  }
  .chars {
    margin-left: auto;
  }
  .prompt-actions {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 8px 8px 0;
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
    min-width: 180px;
    max-height: 240px;
    overflow: auto;
    border-radius: 10px;
    border: 1px solid var(--glass-border);
    background: rgba(12, 16, 26, 0.98);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    display: flex;
    flex-direction: column;
    padding: 4px;
  }
  .move-menu button {
    appearance: none;
    border: 0;
    background: transparent;
    color: var(--glass-text);
    text-align: left;
    padding: 8px 10px;
    border-radius: 8px;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .move-menu button:hover {
    background: rgba(255, 255, 255, 0.06);
  }
  .empty-main {
    padding: 32px 16px;
    text-align: center;
    color: var(--glass-text-dim);
    font-size: 13px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    align-items: center;
  }
  .control-btn,
  .btn-ghost {
    appearance: none;
    border: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.04);
    color: var(--glass-text);
    border-radius: 10px;
    padding: 10px 14px;
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .control-btn.sm,
  .btn-ghost.sm {
    padding: 6px 10px;
    font-size: 12px;
  }
  .btn-ghost {
    border-radius: 999px;
  }
  .control-btn:hover,
  .btn-ghost:hover {
    background: rgba(255, 255, 255, 0.08);
  }
  .control-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .error {
    margin: 0 0 12px;
    color: #ffb4b4;
    font-size: 13px;
  }
  .error.banner {
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid rgba(255, 120, 120, 0.35);
    background: rgba(80, 20, 20, 0.35);
  }
  .lock {
    margin-left: 6px;
  }
</style>
