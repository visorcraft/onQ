import { expect, it } from 'vitest';
import { mount, tick, unmount } from 'svelte';
import BodyEditor from './BodyEditor.svelte';

function setup(overrides: Partial<{
  body: string;
  mode: 'edit' | 'preview';
  locked: boolean;
  busy: boolean;
  isDraft: boolean;
  historyEntries: { path: string; timestamp: string; bytes: number }[];
  showHistory: boolean;
  onBodyInput: (value: string) => void;
  onModeChange: (mode: 'edit' | 'preview') => void;
  onToggleHistory: () => void;
  onRestoreHistory: (path: string) => void;
}> = {}) {
  document.body.replaceChildren();
  const target = document.createElement('div');
  document.body.append(target);
  const props = {
    body: '',
    mode: 'edit' as const,
    locked: false,
    busy: false,
    isDraft: true,
    historyEntries: [],
    showHistory: false,
    onBodyInput: () => {},
    onModeChange: () => {},
    onToggleHistory: () => {},
    onRestoreHistory: () => {},
    ...overrides,
  };
  return mount(BodyEditor, { target, props });
}

it('renders one gutter line number per body line', async () => {
  const component = setup({ body: 'one\ntwo\nthree' });
  await tick();

  const numbers = Array.from(document.querySelectorAll('.gutter span')).map((el) =>
    el.textContent?.trim(),
  );
  expect(numbers).toEqual(['1', '2', '3']);
  await unmount(component);
});

it('emits onModeChange when the Preview segment is clicked', async () => {
  const modes: string[] = [];
  const component = setup({ onModeChange: (m) => modes.push(m) });
  await tick();

  const preview = Array.from(document.querySelectorAll<HTMLButtonElement>('.segment')).find(
    (b) => b.textContent?.trim() === 'Preview',
  );
  if (!preview) throw new Error('Preview segment not found');
  preview.click();

  expect(modes).toEqual(['preview']);
  await unmount(component);
});

it('renders the preview pane instead of the textarea in preview mode', async () => {
  const component = setup({ body: 'hello\nworld', mode: 'preview' });
  await tick();

  expect(document.querySelector('textarea.body')).toBeNull();
  const pane = document.querySelector('.preview-pane');
  expect(pane).not.toBeNull();
  expect(pane?.textContent).toContain('hello');
  expect(pane?.textContent).toContain('world');
  await unmount(component);
});

it('shows the history dropdown and emits onRestoreHistory', async () => {
  const restored: string[] = [];
  const component = setup({
    isDraft: false,
    showHistory: true,
    historyEntries: [{ path: '.onq/history/snap-1', timestamp: '2026-07-24 08:00', bytes: 42 }],
    onRestoreHistory: (p) => restored.push(p),
  });
  await tick();

  const historyBtn = document.querySelector('.history-btn');
  expect(historyBtn).not.toBeNull();
  expect(historyBtn?.textContent).toContain('(1)');

  const restore = Array.from(document.querySelectorAll<HTMLButtonElement>('.history-list button')).find(
    (b) => b.textContent?.trim() === 'Restore',
  );
  if (!restore) throw new Error('Restore button not found');
  restore.click();

  expect(restored).toEqual(['.onq/history/snap-1']);
  await unmount(component);
});

it('hides the history button for drafts', async () => {
  const component = setup({
    isDraft: true,
    historyEntries: [{ path: '.onq/history/snap-1', timestamp: '2026-07-24 08:00', bytes: 42 }],
  });
  await tick();

  expect(document.querySelector('.history-btn')).toBeNull();
  await unmount(component);
});
