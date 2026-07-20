import { writable } from 'svelte/store';

/** Top-level app surfaces (full pages, not modals). */
export type AppView = 'home' | 'settings' | 'about' | 'licenses' | 'credits';

export const appView = writable<AppView>('home');

export function navigate(view: AppView) {
  appView.set(view);
}
