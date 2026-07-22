import js from '@eslint/js';
import globals from 'globals';
import ts from '@typescript-eslint/eslint-plugin';
import tsParser from '@typescript-eslint/parser';
import svelte from 'eslint-plugin-svelte';
import svelteParser from 'svelte-eslint-parser';
import storybook from 'eslint-plugin-storybook';

export default [
  js.configs.recommended,
  {
    files: ['**/*.ts', '**/*.svelte.ts'],
    languageOptions: {
      parser: tsParser,
      parserOptions: { sourceType: 'module' },
      globals: {
        ...globals.browser,
        // Svelte 5 runes — used by `.svelte.ts` modules that expose
        // reactive state to non-component callers.
        $state: 'readonly',
        $derived: 'readonly',
        $effect: 'readonly',
        $props: 'readonly',
        $bindable: 'readonly',
      },
    },
    plugins: { '@typescript-eslint': ts },
    rules: { ...ts.configs.recommended.rules },
  },
  // Unit tests may use Node built-ins (fs, child_process, process).
  {
    files: ['**/*.{test,spec}.ts'],
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
    },
  },
  ...svelte.configs['flat/recommended'],
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parser: svelteParser,
      parserOptions: { parser: tsParser, sourceType: 'module' },
      globals: { ...globals.browser },
    },
  },
  {
    ignores: [
      'dist/',
      'docs-site/.vitepress/dist/',
      'node_modules/',
      'src-tauri/',
      'target/',
      'storybook-static/',
    ],
  },
  ...storybook.configs['flat/recommended'],
];
