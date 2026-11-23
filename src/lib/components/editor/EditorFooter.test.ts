import { afterEach, describe, expect, it } from 'vitest';
import { mount, tick, unmount } from 'svelte';
import EditorFooter from './EditorFooter.svelte';
import { setLocale } from '$lib/i18n';

afterEach(() => {
  document.body.replaceChildren();
  setLocale('en');
});

function mountFooter(props: { charCount: number }) {
  const target = document.createElement('div');
  document.body.append(target);
  const component = mount(EditorFooter, {
    target,
    props: {
      charCount: props.charCount,
      isDraft: false,
      locked: false,
      busy: false,
      saving: false,
      copied: false,
      onDelete: () => {},
      onCancel: () => {},
      onCopy: () => {},
      onSave: () => {},
    },
  });
  return { component, target };
}

describe('EditorFooter', () => {
  it('formats the character count in the selected app locale (de)', async () => {
    setLocale('de');
    const { component, target } = mountFooter({ charCount: 12345 });
    await tick();

    expect(target.querySelector('.char-num')?.textContent?.trim()).toBe('12.345');

    await unmount(component);
  });

  it('formats the character count in the selected app locale (en)', async () => {
    setLocale('en');
    const { component, target } = mountFooter({ charCount: 12345 });
    await tick();

    expect(target.querySelector('.char-num')?.textContent?.trim()).toBe('12,345');

    await unmount(component);
  });
});
