import { afterEach, describe, expect, it } from 'vitest';
import { mount, tick, unmount } from 'svelte';
import TutorialOverlay from './TutorialOverlay.svelte';
import { resolve, setLocale } from '$lib/i18n';

afterEach(() => {
  document.body.replaceChildren();
  setLocale('en');
});

function mountOverlay(props: {
  step: number;
  totalSteps: number;
  title: string;
  body: string;
}) {
  const target = document.createElement('div');
  document.body.append(target);
  const component = mount(TutorialOverlay, {
    target,
    props: {
      step: props.step,
      totalSteps: props.totalSteps,
      title: props.title,
      body: props.body,
    },
  });
  return { component, target };
}

describe('TutorialOverlay', () => {
  it('localizes the dialog, eyebrow, skip and next labels', async () => {
    setLocale('de');
    const { component, target } = mountOverlay({
      step: 2,
      totalSteps: 4,
      title: 'T',
      body: 'B',
    });
    await tick();

    const dialog = target.querySelector('[role="dialog"]');
    expect(dialog?.getAttribute('aria-label')).toBe(
      resolve('de', 'tutorial.dialogAria'),
    );

    const eyebrow = target.querySelector('.eyebrow');
    expect(eyebrow?.textContent?.trim()).toBe(
      resolve('de', 'tutorial.stepOf', { current: 2, total: 4 }),
    );
    expect(eyebrow?.textContent?.trim()).not.toBe('Step 2 of 4');

    const skip = target.querySelector('.skip');
    expect(skip?.textContent?.trim()).toBe(resolve('de', 'tutorial.skip'));

    const next = target.querySelector('.next');
    expect(next?.textContent?.trim()).toBe(resolve('de', 'common.next'));

    await unmount(component);
  });

  it('shows the finish label on the last step', async () => {
    setLocale('de');
    const { component, target } = mountOverlay({
      step: 4,
      totalSteps: 4,
      title: 'T',
      body: 'B',
    });
    await tick();

    const next = target.querySelector('.next');
    expect(next?.textContent?.trim()).toBe(resolve('de', 'tutorial.finish'));
    expect(next?.textContent?.trim()).not.toBe('Finish');

    await unmount(component);
  });
});
