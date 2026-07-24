import { expect, it } from 'vitest';
import { mount, tick, unmount } from 'svelte';
import TagInput from './TagInput.svelte';

function setup(props: {
  tags: string[];
  knownTags?: string[];
  suggestions?: string[];
  disabled?: boolean;
  onChange: (tags: string[]) => void;
  onSuggestionAccepted?: () => void;
  onDismissSuggestions?: () => void;
}) {
  document.body.replaceChildren();
  const target = document.createElement('div');
  document.body.append(target);
  return mount(TagInput, { target, props });
}

function draftInput(): HTMLInputElement {
  const input = document.querySelector('input.tag-draft');
  if (!(input instanceof HTMLInputElement)) throw new Error('draft input not found');
  return input;
}

async function typeDraft(value: string) {
  const input = draftInput();
  input.value = value;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  await tick();
}

it('commits the draft on Enter', async () => {
  const changes: string[][] = [];
  const component = setup({ tags: [], onChange: (t) => changes.push(t) });
  await tick();

  await typeDraft('release');
  draftInput().dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));

  expect(changes).toEqual([['release']]);
  await unmount(component);
});

it('commits comma-separated drafts as multiple tags on Enter', async () => {
  const changes: string[][] = [];
  const component = setup({ tags: [], onChange: (t) => changes.push(t) });
  await tick();

  await typeDraft('bug fix, build');
  draftInput().dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));

  expect(changes).toEqual([['bug fix', 'build']]);
  await unmount(component);
});

it('does not call onChange for a case-insensitive duplicate', async () => {
  const changes: string[][] = [];
  const component = setup({ tags: ['Mongrel'], onChange: (t) => changes.push(t) });
  await tick();

  await typeDraft('mongrel');
  draftInput().dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));

  expect(changes).toEqual([]);
  await unmount(component);
});

it('removes a chip via its remove button', async () => {
  const changes: string[][] = [];
  const component = setup({ tags: ['mongrel', 'release'], onChange: (t) => changes.push(t) });
  await tick();

  const removeButtons = document.querySelectorAll<HTMLButtonElement>('.chip .x');
  expect(removeButtons.length).toBe(2);
  removeButtons[0].click();

  expect(changes).toEqual([['release']]);
  await unmount(component);
});

it('removes the last chip on Backspace when the draft is empty', async () => {
  const changes: string[][] = [];
  const component = setup({ tags: ['mongrel', 'release'], onChange: (t) => changes.push(t) });
  await tick();

  draftInput().dispatchEvent(new KeyboardEvent('keydown', { key: 'Backspace', bubbles: true }));

  expect(changes).toEqual([['mongrel']]);
  await unmount(component);
});

it('Tab accepts the first suggestion and notifies the parent', async () => {
  const changes: string[][] = [];
  let accepted = 0;
  const component = setup({
    tags: [],
    suggestions: ['bug fix', 'build'],
    onChange: (t) => changes.push(t),
    onSuggestionAccepted: () => (accepted += 1),
  });
  await tick();

  draftInput().dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true }));

  expect(changes).toEqual([['bug fix']]);
  expect(accepted).toBe(1);
  await unmount(component);
});

it('lets Escape bubble when the dropdown is closed and there are no suggestions', async () => {
  const component = setup({ tags: ['mongrel'], onChange: () => {} });
  await tick();

  let bubbleHits = 0;
  const onBodyKeydown = () => (bubbleHits += 1);
  document.body.addEventListener('keydown', onBodyKeydown);

  const event = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true });
  draftInput().dispatchEvent(event);

  document.body.removeEventListener('keydown', onBodyKeydown);
  expect(event.defaultPrevented).toBe(false);
  expect(bubbleHits).toBe(1);
  await unmount(component);
});

it('dropdown lists known tags filtered by the draft and excluding added tags', async () => {
  const changes: string[][] = [];
  const component = setup({
    tags: ['mongrel'],
    knownTags: ['mongrel', 'release', 'aur'],
    onChange: (t) => changes.push(t),
  });
  await tick();

  const chevron = document.querySelector<HTMLButtonElement>('button.chevron');
  if (!chevron) throw new Error('chevron not found');
  chevron.click();
  await tick();

  let options = Array.from(document.querySelectorAll('.tag-dropdown .tag-option')).map(
    (el) => el.textContent?.trim(),
  );
  expect(options).toEqual(['release', 'aur']);

  await typeDraft('au');
  options = Array.from(document.querySelectorAll('.tag-dropdown .tag-option')).map(
    (el) => el.textContent?.trim(),
  );
  expect(options).toEqual(['aur']);

  const option = document.querySelector<HTMLButtonElement>('.tag-dropdown .tag-option');
  if (!option) throw new Error('option not found');
  option.click();

  expect(changes).toEqual([['mongrel', 'aur']]);
  await unmount(component);
});
