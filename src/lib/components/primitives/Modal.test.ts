import { afterEach, describe, expect, it } from 'vitest';
import { createRawSnippet, mount, tick, unmount } from 'svelte';
import Modal from './Modal.svelte';
import { resolve, setLocale } from '$lib/i18n';
// jsdom does not implement the Web Animations API; stub it so Svelte's
// exit transitions complete and the dialog is removed from the DOM.
Element.prototype.animate = function () {
  const animation: {
    currentTime: number;
    playState: string;
    effect: unknown;
    onfinish: (() => void) | null;
    cancel: () => void;
  } = {
    currentTime: 0,
    playState: 'running',
    effect: null,
    onfinish: null,
    cancel() {
      animation.playState = 'finished';
    },
  };
  queueMicrotask(() => {
    if (animation.playState === 'finished') return;
    animation.playState = 'finished';
    animation.onfinish?.();
  });
  return animation as Animation;
} as typeof Element.prototype.animate;

afterEach(() => {
  document.body.replaceChildren();
  setLocale('en');
});

function mountModal(props: {
  open: boolean;
  title?: string;
  children?: import('svelte').Snippet;
}) {
  const target = document.createElement('div');
  document.body.append(target);
  const component = mount(Modal, {
    target,
    props: { open: props.open, title: props.title ?? '', children: props.children },
  });
  return { component, target };
}

async function flush() {
  await tick();
  await new Promise((resolve) => setTimeout(resolve, 0));
  await tick();
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

  it('moves focus to the close button when opened with a title', async () => {
    const { component, target } = mountModal({ open: true, title: 'Confirm' });
    await flush();

    const closeButton = target.querySelector('.close');
    expect(closeButton).not.toBeNull();
    expect(document.activeElement).toBe(closeButton);

    await unmount(component);
  });

  it('focuses the panel itself when opened without a title', async () => {
    const { component, target } = mountModal({ open: true, title: '' });
    await flush();

    const panel = target.querySelector('[role="dialog"]');
    expect(panel).not.toBeNull();
    expect(document.activeElement).toBe(panel);

    await unmount(component);
  });

  it('restores focus to the triggering element on Escape', async () => {
    const trigger = document.createElement('button');
    document.body.append(trigger);
    trigger.focus();

    const { component, target } = mountModal({ open: true, title: 'Confirm' });
    await flush();

    const panel = target.querySelector('[role="dialog"]') as HTMLElement;
    panel.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }),
    );
    await flush();

    expect(target.querySelector('[role="dialog"]')).toBeNull();
    expect(document.activeElement).toBe(trigger);

    await unmount(component);
  });

  it('wraps Tab from the last focusable element back to the first', async () => {
    const children = createRawSnippet(() => ({
      render: () => '<button class="child-focusable">Child</button>',
    }));
    const { component, target } = mountModal({ open: true, title: 'Confirm', children });
    await flush();

    const closeButton = target.querySelector<HTMLElement>('.close');
    const childButton = target.querySelector<HTMLElement>('.child-focusable');
    expect(closeButton).not.toBeNull();
    expect(childButton).not.toBeNull();

    childButton!.focus();
    childButton!.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true }),
    );

    expect(document.activeElement).toBe(closeButton);

    await unmount(component);
  });
});
