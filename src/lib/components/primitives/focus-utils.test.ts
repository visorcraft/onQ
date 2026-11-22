import { afterEach, describe, expect, it } from 'vitest';
import { captureFocus, focusableWithin, handleDialogTab } from './focus-utils';

function keydown(key: string, opts: { shiftKey?: boolean } = {}): KeyboardEvent {
  return new KeyboardEvent('keydown', {
    key,
    shiftKey: opts.shiftKey ?? false,
    bubbles: true,
    cancelable: true,
  });
}

function mountRoot(html: string): HTMLElement {
  const root = document.createElement('div');
  root.innerHTML = html;
  document.body.append(root);
  return root;
}

afterEach(() => {
  document.body.replaceChildren();
});

describe('focusableWithin', () => {
  it('returns enabled focusable elements in document order', () => {
    const root = mountRoot(`
      <button class="a">A</button>
      <button class="b" disabled>B</button>
      <a class="c" href="#">C</a>
      <input class="d" />
      <input class="e" disabled />
      <span class="f" tabindex="-1">F</span>
      <span class="g" tabindex="0">G</span>
    `);

    const focusable = focusableWithin(root);

    expect(focusable.map((el) => el.className)).toEqual(['a', 'c', 'd', 'g']);
  });
});

describe('handleDialogTab', () => {
  it('wraps Tab from the last focusable element back to the first', () => {
    const root = mountRoot(`
      <button class="first">First</button>
      <button class="last">Last</button>
    `);
    const first = root.querySelector<HTMLElement>('.first')!;
    const last = root.querySelector<HTMLElement>('.last')!;
    last.focus();

    const event = keydown('Tab');
    handleDialogTab(root, event);

    expect(event.defaultPrevented).toBe(true);
    expect(document.activeElement).toBe(first);
  });

  it('wraps Shift+Tab from the first focusable element back to the last', () => {
    const root = mountRoot(`
      <button class="first">First</button>
      <button class="last">Last</button>
    `);
    const first = root.querySelector<HTMLElement>('.first')!;
    const last = root.querySelector<HTMLElement>('.last')!;
    first.focus();

    const event = keydown('Tab', { shiftKey: true });
    handleDialogTab(root, event);

    expect(event.defaultPrevented).toBe(true);
    expect(document.activeElement).toBe(last);
  });

  it('does not wrap when focus is on a middle element', () => {
    const root = mountRoot(`
      <button class="first">First</button>
      <button class="middle">Middle</button>
      <button class="last">Last</button>
    `);
    const middle = root.querySelector<HTMLElement>('.middle')!;
    middle.focus();

    const event = keydown('Tab');
    handleDialogTab(root, event);

    expect(event.defaultPrevented).toBe(false);
    expect(document.activeElement).toBe(middle);
  });

  it('ignores non-Tab keys', () => {
    const root = mountRoot(`<button class="only">Only</button>`);
    const only = root.querySelector<HTMLElement>('.only')!;
    only.focus();

    const event = keydown('Enter');
    handleDialogTab(root, event);

    expect(event.defaultPrevented).toBe(false);
    expect(document.activeElement).toBe(only);
  });
});

describe('captureFocus', () => {
  it('refocuses the captured element on restore', () => {
    const button = document.createElement('button');
    document.body.append(button);
    button.focus();
    const restore = captureFocus();

    const other = document.createElement('button');
    document.body.append(other);
    other.focus();

    restore();

    expect(document.activeElement).toBe(button);
  });

  it('is a no-op when the captured element is disconnected', () => {
    const button = document.createElement('button');
    document.body.append(button);
    button.focus();
    const restore = captureFocus();

    button.remove();

    expect(() => restore()).not.toThrow();
    expect(document.activeElement).not.toBe(button);
  });
});
