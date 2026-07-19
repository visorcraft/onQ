import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { fileURLToPath, URL } from 'node:url';

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL('./src/lib', import.meta.url)),
    },
    conditions: ['browser'],
  },
  server: { port: 5173, strictPort: true },
  build: { target: 'es2022', sourcemap: true },
  test: {
    include: ['src/**/*.{test,spec}.ts'],
    environment: 'jsdom',
  },
});
