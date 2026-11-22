import { afterEach, describe, expect, it } from 'vitest';
import { mount, tick, unmount } from 'svelte';
import Pager from './Pager.svelte';
import { resolve, setLocale } from '$lib/i18n';

afterEach(() => {
  document.body.replaceChildren();
  setLocale('en');
});

function mountPager(props: {
  page: number;
  pageCount: number;
  ariaLabel?: string;
}) {
  const target = document.createElement('div');
  document.body.append(target);
  const component = mount(Pager, {
    target,
    props: {
      page: props.page,
      pageCount: props.pageCount,
      ariaLabel: props.ariaLabel,
      onPage: () => {},
    },
  });
  return { component, target };
}

describe('Pager', () => {
  it('localizes the nav label and prev/next buttons', async () => {
    setLocale('de');
    const { component, target } = mountPager({ page: 2, pageCount: 3 });
    await tick();

    const nav = target.querySelector('nav');
    expect(nav).not.toBeNull();
    expect(nav?.getAttribute('aria-label')).toBe(
      resolve('de', 'pager.label'),
    );
    expect(nav?.getAttribute('aria-label')).not.toBe('Pagination');

    const buttons = Array.from(target.querySelectorAll('button'));
    expect(buttons).toHaveLength(2);
    expect(buttons[0]?.getAttribute('aria-label')).toBe(
      resolve('de', 'pager.previous'),
    );
    expect(buttons[1]?.getAttribute('aria-label')).toBe(
      resolve('de', 'pager.next'),
    );
    expect(buttons[0]?.getAttribute('aria-label')).not.toBe('Previous page');
    expect(buttons[1]?.getAttribute('aria-label')).not.toBe('Next page');

    await unmount(component);
  });

  it('lets an explicit ariaLabel override win', async () => {
    setLocale('de');
    const { component, target } = mountPager({
      page: 2,
      pageCount: 3,
      ariaLabel: 'Custom label',
    });
    await tick();

    expect(target.querySelector('nav')?.getAttribute('aria-label')).toBe(
      'Custom label',
    );

    await unmount(component);
  });

  it('re-renders the label when the locale changes', async () => {
    setLocale('en');
    const { component, target } = mountPager({ page: 2, pageCount: 3 });
    await tick();

    await setLocale('fr');
    await tick();

    expect(target.querySelector('nav')?.getAttribute('aria-label')).toBe(
      resolve('fr', 'pager.label'),
    );

    await unmount(component);
  });

  it('renders no nav for a single page and disables edge buttons', async () => {
    const single = mountPager({ page: 1, pageCount: 1 });
    await tick();
    expect(single.target.querySelector('nav')).toBeNull();
    await unmount(single.component);

    const first = mountPager({ page: 1, pageCount: 3 });
    await tick();
    const firstButtons = Array.from(first.target.querySelectorAll('button'));
    expect(firstButtons[0]).toHaveProperty('disabled', true);
    expect(firstButtons[1]).toHaveProperty('disabled', false);
    await unmount(first.component);

    const last = mountPager({ page: 3, pageCount: 3 });
    await tick();
    const lastButtons = Array.from(last.target.querySelectorAll('button'));
    expect(lastButtons[0]).toHaveProperty('disabled', false);
    expect(lastButtons[1]).toHaveProperty('disabled', true);
    await unmount(last.component);
  });
});
