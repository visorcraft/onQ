import { afterEach, describe, expect, it } from 'vitest';
import { mount, tick, unmount } from 'svelte';
import Modal from './Modal.svelte';
import { resolve, setLocale } from '$lib/i18n';

afterEach(() => {
  document.body.replaceChildren();
  setLocale('en');
});

function mountModal(props: { open: boolean; title?: string }) {
  const target = document.createElement('div');
  document.body.append(target);
  const component = mount(Modal, {
    target,
    props: { open: props.open, title: props.title ?? '' },
  });
  return { component, target };
}

describe('Modal', () => {
  it('localizes the backdrop and dialog labels', async () => {
    setLocale('de');
    const { component, target } = mountModal({ open: true, title: '' });
    await tick();

    const backdrop = target.querySelector('.backdrop');
    expect(backdrop?.getAttribute('aria-label')).toBe(
      resolve('de', 'common.closeDialog'),
    );
    expect(backdrop?.getAttribute('aria-label')).not.toBe('Close dialog');

    const dialog = target.querySelector('[role="dialog"]');
    expect(dialog?.getAttribute('aria-label')).toBe(
      resolve('de', 'common.dialog'),
    );

    await unmount(component);
  });

  it('keeps an explicit title and localizes the close button', async () => {
    setLocale('de');
    const { component, target } = mountModal({ open: true, title: 'Confirm' });
    await tick();

    const dialog = target.querySelector('[role="dialog"]');
    expect(dialog?.getAttribute('aria-label')).toBe('Confirm');

    const closeButton = target.querySelector('.close');
    expect(closeButton?.getAttribute('aria-label')).toBe(
      resolve('de', 'common.close'),
    );
    expect(closeButton?.getAttribute('aria-label')).not.toBe('Close');

    await unmount(component);
  });
});
