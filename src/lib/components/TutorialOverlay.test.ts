import { expect, it, vi } from 'vitest';
import { mount, tick, unmount } from 'svelte';

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

import TutorialOverlay from './TutorialOverlay.svelte';
import { tutorialStep, tutorialVisible } from '$lib/stores/tutorial';

function findButton(label: string): HTMLButtonElement {
  const button = Array.from(document.querySelectorAll('button')).find(
    (candidate) => candidate.textContent?.trim() === label,
  );
  if (!(button instanceof HTMLButtonElement)) {
    throw new Error(`Button not found: ${label}`);
  }
  return button;
}

it('walks through all four tutorial steps and persists completion', async () => {
  document.body.replaceChildren();
  const target = document.createElement('div');
  target.id = 'target';
  document.body.append(target);
  invokeMock.mockReset();
  invokeMock.mockResolvedValue(undefined);
  tutorialStep.set(0);
  tutorialVisible.set(true);

  const component = mount(TutorialOverlay, { target });
  await tick();

  const dialog = document.querySelector('[role="dialog"]');
  expect(dialog).not.toBeNull();
  expect(dialog?.textContent).toContain('Open your prompt palette');
  expect(findButton('Back').disabled).toBe(true);

  dialog?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }));
  await tick();
  expect(dialog?.textContent).toContain('Create your first prompt');

  dialog?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true }));
  await tick();
  expect(dialog?.textContent).toContain('Open your prompt palette');

  findButton('Next').click();
  findButton('Next').click();
  findButton('Next').click();
  await tick();
  expect(dialog?.textContent).toContain('Your vault stays encrypted');
  expect(dialog?.textContent).toContain('Master password or system keychain');
  expect(dialog?.querySelector('[aria-label="Recovery phrase preview"]')).toBeNull();

  findButton('Done').click();
  await vi.waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith('set_app_setting', {
      key: 'tutorial_completed',
      value: 'true',
    });
    expect(document.querySelector('[role="dialog"]')).toBeNull();
  });

  await unmount(component);
});

it('dismisses with Escape when completion cannot be persisted', async () => {
  document.body.replaceChildren();
  const target = document.createElement('div');
  document.body.append(target);
  invokeMock.mockReset();
  invokeMock.mockRejectedValueOnce(new Error('write failed'));
  tutorialStep.set(0);
  tutorialVisible.set(true);

  const component = mount(TutorialOverlay, { target });
  await tick();

  document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));

  await vi.waitFor(() => {
    expect(document.querySelector('[role="dialog"]')).toBeNull();
  });

  await unmount(component);
});

it('dismisses on outside interaction when completion cannot be persisted', async () => {
  document.body.replaceChildren();
  const target = document.createElement('div');
  document.body.append(target);
  invokeMock.mockReset();
  invokeMock.mockRejectedValueOnce(new Error('write failed'));
  tutorialStep.set(0);
  tutorialVisible.set(true);

  const component = mount(TutorialOverlay, { target });
  await tick();
  await new Promise((resolve) => setTimeout(resolve, 5));

  document.body.dispatchEvent(
    new PointerEvent('pointerdown', { bubbles: true, clientX: -1, clientY: -1 }),
  );

  await vi.waitFor(() => {
    expect(document.querySelector('[role="dialog"]')).toBeNull();
  });

  await unmount(component);
});
