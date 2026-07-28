import { defineConfig } from '@playwright/test';
import process from 'node:process';

if (process.env.NO_COLOR !== undefined) {
  process.env.FORCE_COLOR ??= '0';
  delete process.env.NO_COLOR;
}

export default defineConfig({
  testDir: './tests',
  use: {
    baseURL: 'http://localhost:5173',
  },
  webServer: {
    command: 'npm run dev',
    port: 5173,
    reuseExistingServer: true,
  },
});
