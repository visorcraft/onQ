import { afterEach, expect, it, vi } from 'vitest';
import { mount, tick, unmount } from 'svelte';

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

import LicensesPage from './LicensesPage.svelte';
import { resolve, setLocale } from '$lib/i18n';

const DOCS = [{ id: 'app', title: 'onQ', subtitle: 'GPL-3.0-only' }];

function mockLicenses(documentText: string | Promise<string>) {
  invokeMock.mockReset();
  invokeMock.mockImplementation((cmd: string) => {
    if (cmd === 'license_docs_cmd') return Promise.resolve(DOCS);
    if (cmd === 'license_document_cmd')
      return documentText instanceof Promise ? documentText : Promise.resolve(documentText);
    return Promise.reject(new Error(`unexpected invoke: ${cmd}`));
  });
}

const mounted: Array<ReturnType<typeof mount>> = [];

function mountPage(locale: 'en' | 'de' = 'en') {
  document.body.replaceChildren();
  setLocale(locale);
  const target = document.createElement('div');
  document.body.append(target);
  const component = mount(LicensesPage, { target });
  mounted.push(component);
  return { component, target };
}

async function waitForBody(target: HTMLElement, expected: string) {
  await vi.waitFor(() => {
    expect(target.querySelector('.licenses-body')?.textContent).toBe(expected);
  });
}

function filterInput(target: HTMLElement): HTMLInputElement {
  const input = target.querySelector('input[type="search"]');
  expect(input).not.toBeNull();
  return input as HTMLInputElement;
}

function clearButton(target: HTMLElement): HTMLButtonElement {
  const button = target.querySelector('.licenses-filter-row button.btn-ghost');
  expect(button).not.toBeNull();
  return button as HTMLButtonElement;
}

function wrapToggleLabel(target: HTMLElement): HTMLElement {
  const label = target.querySelector('.wrap-toggle');
  expect(label).not.toBeNull();
  return label as HTMLElement;
}

afterEach(async () => {
  while (mounted.length > 0) {
    const component = mounted.pop();
    if (component) await unmount(component);
  }
  document.body.replaceChildren();
  setLocale('en');
});

it('renders English chrome and interpolated line count', async () => {
  mockLicenses('alpha\nbeta\ngamma');
  const { target } = mountPage();

  await waitForBody(target, 'alpha\nbeta\ngamma');

  const documentCall = invokeMock.mock.calls.find(
    ([cmd]) => cmd === 'license_document_cmd',
  );
  expect(documentCall).toBeDefined();
  expect(documentCall![1]).toEqual({ id: 'app' });

  expect(target.querySelector('.line-count')?.textContent).toBe('3 lines');
  expect(wrapToggleLabel(target).textContent?.trim()).toBe('Wrap');
  expect(clearButton(target).textContent?.trim()).toBe('Clear');
  expect(target.querySelector('h1')?.textContent?.trim()).toBe('Licenses');
});

it('shows the no-match text when the filter has zero hits', async () => {
  mockLicenses('alpha\nbeta\ngamma');
  const { target } = mountPage();
  await waitForBody(target, 'alpha\nbeta\ngamma');

  const input = filterInput(target);
  input.value = 'zzz';
  input.dispatchEvent(new Event('input', { bubbles: true }));
  await tick();

  expect(target.querySelector('.licenses-body')?.textContent).toBe(
    '(no matching lines)',
  );
});

it('clears the filter with the clear button and restores the body', async () => {
  mockLicenses('alpha\nbeta\ngamma');
  const { target } = mountPage();
  await waitForBody(target, 'alpha\nbeta\ngamma');

  const input = filterInput(target);
  input.value = 'zzz';
  input.dispatchEvent(new Event('input', { bubbles: true }));
  await tick();
  expect(target.querySelector('.licenses-body')?.textContent).toBe(
    '(no matching lines)',
  );

  clearButton(target).click();
  await tick();

  expect(filterInput(target).value).toBe('');
  expect(target.querySelector('.licenses-body')?.textContent).toBe(
    'alpha\nbeta\ngamma',
  );
});

it('renders localized chrome in German', async () => {
  mockLicenses('alpha\nbeta\ngamma');
  const { target } = mountPage('de');
  await waitForBody(target, 'alpha\nbeta\ngamma');

  expect(target.querySelector('.line-count')?.textContent).toBe('3 Zeilen');
  expect(wrapToggleLabel(target).textContent?.trim()).toBe('Zeilenumbruch');
  expect(clearButton(target).textContent?.trim()).toBe('Löschen');
  expect(target.querySelector('.line-count')?.textContent).not.toBe('3 lines');
  expect(wrapToggleLabel(target).textContent?.trim()).not.toBe('Wrap');
  expect(clearButton(target).textContent?.trim()).not.toBe('Clear');

  const input = filterInput(target);
  input.value = 'zzz';
  input.dispatchEvent(new Event('input', { bubbles: true }));
  await tick();

  const body = target.querySelector('.licenses-body')?.textContent;
  expect(body).toBe('(keine passenden Zeilen)');
  expect(body).not.toBe('(no matching lines)');
});

it('shows the localized loading text while the document is unresolved', async () => {
  mockLicenses(new Promise<string>(() => {}));
  const { target } = mountPage();

  await vi.waitFor(() => {
    expect(target.querySelector('.licenses-body')?.textContent).toBe('Loading…');
  });

  setLocale('de');
  await tick();

  const body = target.querySelector('.licenses-body')?.textContent;
  expect(body).toBe(resolve('de', 'common.loading'));
  expect(body).not.toBe('Loading…');
});

it('derives the line count from the loaded document', async () => {
  mockLicenses('alpha');
  const { target } = mountPage();

  await waitForBody(target, 'alpha');

  expect(target.querySelector('.line-count')?.textContent).toBe('1 lines');
});
