import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { mount, tick, unmount } from 'svelte';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

vi.mock('$lib/api/smartFolderVisual', () => ({
  visualToDsl: vi.fn(async (q: { predicates: Array<{ field: string; value: string }> }) =>
    `${q.predicates[0].field}:${q.predicates[0].value}`,
  ),
}));

import SmartFolderVisualBuilder from './SmartFolderVisualBuilder.svelte';
import { setLocale } from '$lib/i18n';

function mountBuilder(props?: { onDslChange?: (dsl: string) => void }) {
  const target = document.createElement('div');
  document.body.append(target);
  const component = mount(SmartFolderVisualBuilder, {
    target,
    props: props ?? {},
  });
  return { component, target };
}

function optionTexts(select: Element): string[] {
  return Array.from(select.querySelectorAll('option')).map((o) =>
    (o.textContent ?? '').trim(),
  );
}

function optionValues(select: Element): string[] {
  return Array.from(select.querySelectorAll('option')).map(
    (o) => (o as HTMLOptionElement).value,
  );
}

async function selectField(fieldValue: string) {
  const fieldSelect = document.querySelectorAll('select')[0] as HTMLSelectElement;
  fieldSelect.value = fieldValue;
  fieldSelect.dispatchEvent(new Event('change', { bubbles: true }));
  await tick();
}

beforeEach(() => {
  document.body.replaceChildren();
  setLocale('en');
});

afterEach(async () => {
  document.body.replaceChildren();
  setLocale('en');
});

describe('SmartFolderVisualBuilder', () => {
  it('shows German labels with raw token values', async () => {
    setLocale('de');
    const { component } = mountBuilder();
    await tick();

    const selects = document.querySelectorAll('select');
    expect(selects.length).toBeGreaterThanOrEqual(2);

    const fieldSelect = selects[0];
    expect(optionTexts(fieldSelect)).toEqual([
      'Tag',
      'Ordner',
      'Favorit',
      'Gesperrt',
      'Text',
    ]);
    expect(optionValues(fieldSelect)).toEqual([
      'tag',
      'folder',
      'favorite',
      'locked',
      'text',
    ]);

    const operatorSelect = selects[1];
    expect(optionTexts(operatorSelect)).toEqual(['ist', 'ist nicht']);
    expect(optionValues(operatorSelect)).toEqual(['is', 'not']);

    await unmount(component);
  });

  it('shows a boolean value select for favorite and locked fields', async () => {
    setLocale('de');
    const { component } = mountBuilder();
    await tick();

    await selectField('favorite');

    const selects = document.querySelectorAll('select');
    expect(selects.length).toBe(3);

    const operatorSelect = selects[1];
    expect(optionTexts(operatorSelect)).toEqual(['ist']);

    const valueSelect = selects[2];
    expect(optionTexts(valueSelect)).toEqual(['wahr', 'falsch']);
    expect(optionValues(valueSelect)).toEqual(['true', 'false']);

    await unmount(component);
  });

  it('shows contains for the text field and renders a text input instead of a select', async () => {
    setLocale('de');
    const { component } = mountBuilder();
    await tick();

    await selectField('text');

    const selects = document.querySelectorAll('select');
    expect(selects.length).toBe(2);

    const operatorSelect = selects[1];
    expect(optionTexts(operatorSelect)).toEqual(['enthält']);
    expect(optionValues(operatorSelect)).toEqual(['contains']);

    expect(document.querySelector('input[type="text"]')).not.toBeNull();

    await unmount(component);
  });

  it('re-renders labels when the locale switches', async () => {
    setLocale('de');
    const { component } = mountBuilder();
    await tick();

    await selectField('favorite');

    setLocale('en');
    await tick();

    const selects = document.querySelectorAll('select');
    expect(optionTexts(selects[0])).toEqual([
      'Tag',
      'Folder',
      'Favorite',
      'Locked',
      'Text',
    ]);

    const operatorSelect = selects[1];
    expect(optionTexts(operatorSelect)).toEqual(['is']);

    const valueSelect = selects[2];
    expect(optionTexts(valueSelect)).toEqual(['true', 'false']);

    await unmount(component);
  });

  it('adds a chip with the raw DSL tokens', async () => {
    const onDslChange = vi.fn();
    const { component } = mountBuilder({ onDslChange });
    await tick();

    await selectField('favorite');

    const valueSelect = document.querySelectorAll('select')[2] as HTMLSelectElement;
    valueSelect.value = 'true';
    valueSelect.dispatchEvent(new Event('change', { bubbles: true }));
    await tick();

    const addButton = document.querySelector('button.control-btn');
    expect(addButton).not.toBeNull();
    (addButton as HTMLButtonElement).click();

    await vi.waitFor(() => {
      expect(onDslChange).toHaveBeenCalledWith('favorite:true');
    });

    const chip = document.querySelector('.chip');
    expect(chip?.textContent).toContain('favorite:true');

    await unmount(component);
  });
});
