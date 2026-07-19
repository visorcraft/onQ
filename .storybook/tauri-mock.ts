// Tauri API mock for Storybook. Intercepts `window.__TAURI_INTERNALS__.invoke`
// so that components which call `@tauri-apps/api/core#invoke` get plausible
// data instead of crashing in the browser preview. Storybook stories supply
// the fixtures they need; this file just dispatches by command name.

interface MockPrompt {
  id: string;
  title: string;
  body: string;
  folder: string | null;
  tags: string[];
  favorite: boolean;
  locked: boolean;
  updated: string;
  char_count: number;
}

interface MockRecent {
  id: string;
  prompt_id: string;
  opened_at: string;
  title: string;
}

const MOCK_PROMPTS: MockPrompt[] = [
  {
    id: 'prompt-1',
    title: 'Refactor a Svelte 5 component',
    body:
      'You are a senior Svelte engineer. Take this component and rewrite it using Svelte 5 runes.\nPreserve the public API but drop any legacy `export let` syntax.',
    folder: 'writing',
    tags: ['writing', 'svelte'],
    favorite: true,
    locked: false,
    updated: '2026-07-18T10:42:00Z',
    char_count: 187,
  },
  {
    id: 'prompt-2',
    title: 'Code review checklist',
    body: 'Review this PR for: correctness, naming, tests, accessibility, performance.',
    folder: 'engineering',
    tags: ['code-review'],
    favorite: false,
    locked: false,
    updated: '2026-07-17T18:11:00Z',
    char_count: 96,
  },
  {
    id: 'prompt-3',
    title: 'Meeting summary',
    body: 'Summarize the meeting into: decisions, action items, open questions.',
    folder: 'meetings',
    tags: ['meetings', 'summary'],
    favorite: false,
    locked: false,
    updated: '2026-07-16T09:00:00Z',
    char_count: 88,
  },
];

const MOCK_RECENT: MockRecent[] = MOCK_PROMPTS.map((p, index) => ({
  id: `recent-${index + 1}`,
  prompt_id: p.id,
  opened_at: p.updated,
  title: p.title,
}));

type TauriPayload = unknown;

function payload<T>(value: T): TauriPayload {
  return value;
}

const handlers: Record<string, (args: Record<string, unknown>) => unknown> = {
  list_prompts: () => payload(MOCK_PROMPTS),
  read_prompt: ({ id }) => {
    const found = MOCK_PROMPTS.find((p) => p.id === id);
    if (!found) throw new Error(`prompt not found: ${id}`);
    return payload(found);
  },
  create_prompt: ({ title }) =>
    payload({
      id: `prompt-${Math.random().toString(36).slice(2, 9)}`,
      title,
      body: '',
      folder: null,
      tags: [],
      favorite: false,
      locked: false,
      updated: new Date().toISOString(),
      char_count: 0,
    }),
  save_prompt: (args) => {
    const id = args['id'] as string;
    const existing = MOCK_PROMPTS.find((p) => p.id === id);
    if (!existing) throw new Error(`prompt not found: ${id}`);
    Object.assign(existing, {
      title: args['title'],
      folder: args['folder'],
      tags: args['tags'],
      favorite: args['favorite'],
      body: args['body'],
      char_count: (args['body'] as string).length,
      updated: new Date().toISOString(),
    });
    return payload(existing);
  },
  delete_prompt: ({ id }: { id: string }) => {
    const index = MOCK_PROMPTS.findIndex((p) => p.id === id);
    if (index >= 0) MOCK_PROMPTS.splice(index, 1);
    return payload(null);
  },
  lock_prompt: ({ id }: { id: string }) => {
    const p = MOCK_PROMPTS.find((x) => x.id === id);
    if (p) p.locked = true;
    return payload(p);
  },
  unlock_prompt: ({ id }: { id: string }) => {
    const p = MOCK_PROMPTS.find((x) => x.id === id);
    if (p) p.locked = false;
    return payload(p);
  },
  get_app_setting: () => payload(null),
  set_app_setting: () => payload(null),
  hybrid_search: () =>
    payload({
      hits: MOCK_PROMPTS.map((p, i) => ({
        id: p.id,
        title: p.title,
        score: 1 - i * 0.1,
        snippet: p.body.slice(0, 80),
      })),
    }),
  list_recent: () => payload(MOCK_RECENT),
  record_open: () => payload(null),
  setup_new_vault: () =>
    payload({
      recovery_phrase:
        'alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima mike november oscar papa quebec romeo sierra tango uniform victor whiskey xray',
    }),
  open_vault: () => payload(null),
  pick_vault_dir: () => payload('/mock/vault/path'),
  existing_vault: () => payload(true),
  is_vault_initialized: () => payload(true),
};

declare global {
  interface Window {
    __TAURI_INTERNALS__?: {
      invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
    };
  }
}

export function installTauriMock(): void {
  if (typeof window === 'undefined') return;
  window.__TAURI_INTERNALS__ = {
    invoke: async (cmd, args = {}) => {
      const handler = handlers[cmd];
      if (!handler) {
        // Unmocked commands resolve to null so the story doesn't crash;
        // we never want a missing mock to take down the whole render.
        console.warn(`[tauri-mock] no handler for command: ${cmd}`);
        return null;
      }
      return handler(args);
    },
  };
}
