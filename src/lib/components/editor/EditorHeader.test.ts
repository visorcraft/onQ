import { afterEach, describe, expect, it } from 'vitest';
import { mount, tick, unmount } from 'svelte';
import EditorHeader from './EditorHeader.svelte';
import { resolve, setLocale } from '$lib/i18n';

afterEach(() => {
  document.body.replaceChildren();
  setLocale('en');
});

function mountHeader(props: {
  isDraft: boolean;
  locked: boolean;
  busy?: boolean;
  favorite?: boolean;
}) {
  const target = document.createElement('div');
  document.body.append(target);
  const component = mount(EditorHeader, {
    target,
    props: {
      isDraft: props.isDraft,
      favorite: props.favorite ?? false,
      locked: props.locked,
      busy: props.busy ?? false,
      onToggleFavorite: () => {},
      onToggleLock: () => {},
      onClose: () => {},
    },
  });
  return { component, target };
}

function lockButton(target: HTMLElement, label: string) {
  const buttons = Array.from(target.querySelectorAll('button.icon-chip'));
  return buttons.find((b) => b.getAttribute('aria-label') === label) ?? null;
}

describe('EditorHeader', () => {
  it('localizes the lock tooltip when unlocked', async () => {
    setLocale('de');
    const { component, target } = mountHeader({ isDraft: false, locked: false });
    await tick();

    const lockLabel = resolve('de', 'editor.lock');
    const lockButtonEl = lockButton(target, lockLabel);
    expect(lockButtonEl).not.toBeNull();
    expect(lockButtonEl?.getAttribute('title')).toBe(lockLabel);
    expect(lockButtonEl?.getAttribute('title')).not.toBe('Lock');

    await unmount(component);
  });

  it('localizes the lock tooltip when locked', async () => {
    setLocale('de');
    const { component, target } = mountHeader({ isDraft: false, locked: true });
    await tick();

    const unlockLabel = resolve('de', 'editor.unlock');
    const lockButtonEl = lockButton(target, unlockLabel);
    expect(lockButtonEl).not.toBeNull();
    expect(lockButtonEl?.getAttribute('aria-label')).toBe(unlockLabel);
    expect(lockButtonEl?.getAttribute('title')).toBe(unlockLabel);
    expect(lockButtonEl?.getAttribute('title')).not.toBe('Unlock');

    await unmount(component);
  });

  it('localizes the title field label', async () => {
    setLocale('de');
    const { component, target } = mountHeader({ isDraft: false, locked: false });
    await tick();

    const label = target.querySelector('.field-label');
    expect(label?.textContent?.trim()).toBe(resolve('de', 'editor.title'));
    expect(label?.textContent?.trim()).not.toBe('Title');

    await unmount(component);

    setLocale('en');
    const { component: componentEn, target: targetEn } = mountHeader({ isDraft: false, locked: false });
    await tick();

    const labelEn = targetEn.querySelector('.field-label');
    expect(labelEn?.textContent?.trim()).toBe('Title');

    await unmount(componentEn);
  });

  it('hides the lock button in draft mode', async () => {
    setLocale('en');
    const { component, target } = mountHeader({ isDraft: true, locked: false });
    await tick();

    const buttons = Array.from(target.querySelectorAll('button'));
    const lockLabel = resolve('en', 'editor.lock');
    expect(buttons.some((b) => b.getAttribute('aria-label') === lockLabel)).toBe(false);

    await unmount(component);
  });
});
