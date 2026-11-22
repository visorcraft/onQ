import { afterEach, describe, expect, it } from 'vitest';
import { mount, tick, unmount } from 'svelte';
import ConfirmDialog from './ConfirmDialog.svelte';
import { setLocale } from '$lib/i18n';
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

function mountConfirm(props: { open: boolean; busy?: boolean }) {
  const target = document.createElement('div');
  document.body.append(target);
  const component = mount(ConfirmDialog, {
    target,
    props: { open: props.open, busy: props.busy ?? false },
  });
  return { component, target };
}

async function flush() {
  await tick();
  await new Promise((resolve) => setTimeout(resolve, 0));
  await tick();
}

describe('ConfirmDialog focus', () => {
  it('focuses the confirm button on open', async () => {
    const { component, target } = mountConfirm({ open: true });
    await flush();

    const confirmButton = target.querySelector('.btn.danger');
    expect(confirmButton).not.toBeNull();
    expect(document.activeElement).toBe(confirmButton);

    await unmount(component);
  });

  it('restores focus to the triggering element on Escape', async () => {
    const trigger = document.createElement('button');
    document.body.append(trigger);
    trigger.focus();

    const { component, target } = mountConfirm({ open: true });
    await flush();

    const panel = target.querySelector('[role="alertdialog"]') as HTMLElement;
    panel.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }),
    );
    await flush();

    expect(target.querySelector('[role="alertdialog"]')).toBeNull();
    expect(document.activeElement).toBe(trigger);

    await unmount(component);
  });

  it('keeps the dialog open on Escape while busy', async () => {
    const { component, target } = mountConfirm({ open: true, busy: true });
    await flush();

    const panel = target.querySelector('[role="alertdialog"]') as HTMLElement;
    panel.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }),
    );
    await flush();

    expect(target.querySelector('[role="alertdialog"]')).not.toBeNull();

    await unmount(component);
  });
});
