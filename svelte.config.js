import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

// No top-level `runes: true` — auto-detect. Local components use runes;
// third-party components from cmdk-sv (which still use Svelte 4 syntax
// like `$$restProps` and `export let`) get compiled in legacy mode.
export default {
  preprocess: vitePreprocess(),
};
