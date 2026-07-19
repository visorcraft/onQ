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
    files: ['**/*.ts'],
    languageOptions: {
      parser: tsParser,
      parserOptions: { sourceType: 'module' },
      globals: { ...globals.browser },
    },
    plugins: { '@typescript-eslint': ts },
    rules: { ...ts.configs.recommended.rules },
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
    ignores: ['dist/', 'node_modules/', 'src-tauri/', 'target/', 'storybook-static/'],
  },
  ...storybook.configs['flat/recommended'],
];
